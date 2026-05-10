//! Suite runner: [`Case`] lists, [`HookFns`], [`RunConfig`], and [`run`].
//!
//! Read this module’s docs top-down: the **execution model** below applies to everything else.
//!
//! # Execution model
//!
//! A **case** is a name plus a `fn(&mut S)` body. [`run`] decides which cases to execute, then
//! invokes hooks from [`HookFns`] between those bodies. Nothing runs until [`run`] is called.
//!
//! ## Selection
//!
//! - [`RunConfig::filter`] **None** → all cases in the slice are selected (in slice order).
//! - [`RunConfig::filter`] **Some(name)** → only the case whose [`Case::name`] equals `name`
//!   (string comparison; the [`cases!`] macro uses `stringify!` for names).
//!
//! If the selected set is **empty**:
//! - and `filter` is **Some** → [`run`] **panics** (`"matched no cases"`).
//! - and `filter` is **None** → [`run`] returns immediately (you passed an empty slice).
//!
//! ## Hook order (after selection is non-empty)
//!
//! 1. `setup_suite` if [`Some`].
//! 2. For each selected case, in order:
//!    - `before_each` if [`Some`].
//!    - Run the case body. If it panics, the panic is caught, `after_each` still runs if [`Some`],
//!      then the original panic is resumed.
//!    - `after_each` if [`Some`].
//! 3. `teardown_suite` if [`Some`].
//!
//! Any hook that is [`None`] is skipped.

mod fail;
use fail::FailReason;
pub use fail::{fail, fail_now};

use regex::Regex;

/// Optional lifecycle callbacks. Each field is [`Some`] with a function to run at that point, or
/// [`None`] to skip.
///
/// Use [`HookFns::default`] or [`HookFns::noop`] when you only want case bodies and no hooks.
#[derive(Clone, Copy, Debug)]
pub struct HookFns<S> {
    /// Runs once after selection, before any case.
    pub setup_suite: Option<fn(&mut S)>,
    /// Runs once after all selected cases finish.
    pub teardown_suite: Option<fn(&mut S)>,
    /// Runs before each selected case.
    pub before_each: Option<fn(&mut S)>,
    /// Runs after each selected case, even if the case panicked.
    pub after_each: Option<fn(&mut S)>,
}

impl<S> Default for HookFns<S> {
    fn default() -> Self {
        Self {
            setup_suite: None,
            teardown_suite: None,
            before_each: None,
            after_each: None,
        }
    }
}

impl<S> HookFns<S> {
    /// Same as [`Default::default`]: every hook is [`None`].
    pub fn noop() -> Self {
        Self::default()
    }
}

/// One named **case**: a stable `&'static str` name (for filtering) and a function pointer.
///
/// Closures that capture nothing coerce to `fn(&mut S)`, which is why the macros can build static
/// slices.
pub struct Case<S> {
    pub name: &'static str,
    pub run: fn(&mut S),
    pub dependencies: &'static [&'static str],
}

impl<S> Case<S> {
    pub const fn new(name: &'static str, run: fn(&mut S)) -> Self {
        Self {
            name,
            run,
            dependencies: &[],
        }
    }
}

/// Controls **which** cases [`run`] executes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunConfig {
    /// When set, only the case whose name matches **exactly** runs.
    pub filter: Option<String>,
    /// When set, only cases whose names appear in this list run.
    pub filters: Vec<String>,
    /// When set, only cases whose names match this glob pattern run (`*` matches any chars).
    pub pattern: Option<String>,
}

impl RunConfig {
    /// No filter: run every case in the slice (subject to the empty-selection rules on [`run`]).
    pub fn all() -> Self {
        Self::default()
    }

    /// Run a single case (by [`Case::name`]) or any case whose name equals `name`.
    pub fn filter(name: impl Into<String>) -> Self {
        Self {
            filter: Some(name.into()),
            filters: Vec::new(),
            pattern: None,
        }
    }

    /// Run only cases whose names appear in `names`.
    pub fn filters(names: impl IntoIterator<Item = String>) -> Self {
        Self {
            filter: None,
            filters: names.into_iter().collect(),
            pattern: None,
        }
    }

    /// Run only cases whose names match the glob `pat` (`*` matches any characters).
    pub fn pattern(pat: impl Into<String>) -> Self {
        Self {
            filter: None,
            filters: Vec::new(),
            pattern: Some(pat.into()),
        }
    }

    /// Reads `--case <pattern>` from `std::env::args()` and returns a config
    /// that filters cases by regex or glob. Falls back to [`RunConfig::all`] if not found.
    ///
    /// If the pattern contains regex metacharacters (`^`, `$`, `.`, `+`, `?`, `(`, `)`,
    /// `[`, `]`, `{`, `|`), it is treated as a regex. Otherwise it is treated as a glob
    /// (`*` matches any characters).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use suitecase::{cases, run, Case, HookFns, RunConfig};
    ///
    /// #[derive(Default)]
    /// struct Counter { n: i32 }
    ///
    /// static CASES: &[Case<Counter>] = cases![Counter, s =>
    ///     test_inc => { s.n += 1; },
    /// ];
    ///
    /// let mut suite = Counter::default();
    /// run(&mut suite, CASES, RunConfig::from_args(), &HookFns::default());
    /// ```
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        for i in 0..args.len() {
            if args[i] == "--case" && i + 1 < args.len() {
                return Self::pattern(&args[i + 1]);
            }
        }
        Self::all()
    }
}

/// Run the selected cases on `suite` using `hooks` for lifecycle callbacks.
///
/// See the [module-level **Execution model** section](crate::suite#execution-model) for ordering and
/// panic behavior.
pub fn run<S>(suite: &mut S, cases: &[Case<S>], config: RunConfig, hooks: &HookFns<S>) {
    run_hooks_with_output(
        suite,
        cases,
        config,
        |s| {
            if let Some(f) = hooks.setup_suite {
                f(s);
            }
        },
        |s| {
            if let Some(f) = hooks.teardown_suite {
                f(s);
            }
        },
        |s| {
            if let Some(f) = hooks.before_each {
                f(s);
            }
        },
        |s| {
            if let Some(f) = hooks.after_each {
                f(s);
            }
        },
    );
}

fn run_hooks_with_output<S, FS, FT, FB, FA>(
    suite: &mut S,
    cases: &[Case<S>],
    config: RunConfig,
    mut setup_suite: FS,
    mut teardown_suite: FT,
    mut before_each: FB,
    mut after_each: FA,
) where
    FS: FnMut(&mut S),
    FT: FnMut(&mut S),
    FB: FnMut(&mut S),
    FA: FnMut(&mut S),
{
    let has_filter =
        config.filter.is_some() || !config.filters.is_empty() || config.pattern.is_some();

    let selected: Vec<&Case<S>> = if !has_filter {
        cases.iter().collect()
    } else {
        let mut names: std::collections::HashSet<&str> = std::collections::HashSet::new();

        if let Some(ref f) = config.filter {
            names.insert(f.as_str());
        }
        for f in &config.filters {
            names.insert(f.as_str());
        }

        let direct: Vec<&Case<S>> = cases
            .iter()
            .filter(|c| {
                if names.contains(c.name) {
                    return true;
                }
                if let Some(ref pat) = config.pattern {
                    return pattern_match(pat, c.name);
                }
                false
            })
            .collect();

        let mut resolved: Vec<&Case<S>> = Vec::new();
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        let mut visiting: std::collections::HashSet<&str> = std::collections::HashSet::new();
        let all_names: std::collections::HashMap<&str, &Case<S>> =
            cases.iter().map(|c| (c.name, c)).collect();

        for case in &direct {
            resolve_deps(case, &all_names, &mut resolved, &mut seen, &mut visiting);
        }

        let resolved_names: std::collections::HashSet<&str> =
            resolved.iter().map(|c| c.name).collect();

        let mut in_degree: std::collections::HashMap<&str, usize> =
            resolved_names.iter().map(|n| (*n, 0)).collect();

        for case in &resolved {
            for dep in case.dependencies {
                if resolved_names.contains(*dep) {
                    *in_degree.entry(case.name).or_insert(0) += 1;
                }
            }
        }

        let order_map: std::collections::HashMap<&str, usize> =
            cases.iter().enumerate().map(|(i, c)| (c.name, i)).collect();

        let mut result: Vec<&Case<S>> = Vec::new();
        let mut resolved_case_map: std::collections::HashMap<&str, &Case<S>> =
            resolved.iter().map(|c| (c.name, *c)).collect();

        loop {
            let mut available: Vec<&str> = in_degree
                .iter()
                .filter(|(_, deg)| **deg == 0)
                .map(|(&name, _)| name)
                .collect();
            if available.is_empty() {
                break;
            }
            available.sort_by_key(|n| order_map.get(n).copied().unwrap_or(0));
            let next = available[0];
            in_degree.remove(next);
            if let Some(case) = resolved_case_map.remove(next) {
                result.push(case);
            }
            for case in resolved_case_map.values() {
                for dep in case.dependencies {
                    if *dep == next {
                        if let Some(deg) = in_degree.get_mut(case.name) {
                            *deg = deg.saturating_sub(1);
                        }
                    }
                }
            }
        }

        result
    };

    if selected.is_empty() {
        assert!(!has_filter, "suitecase: filter matched no cases");
        return;
    }

    setup_suite(suite);
    let mut first_panic: Option<Box<dyn std::any::Any + Send>> = None;
    let mut fail_msg: Option<String> = None;
    let mut fail_now_msg: Option<String> = None;
    let mut failed_names: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut skipped_names: std::collections::HashSet<&str> = std::collections::HashSet::new();

    for case in &selected {
        let deps_failed = case
            .dependencies
            .iter()
            .any(|d| failed_names.contains(*d) || skipped_names.contains(*d));

        if deps_failed {
            skipped_names.insert(case.name);
            println!("⊘ {} (skipped, dependency failed)", case.name);
            continue;
        }

        println!("▶ {}", case.name);
        let start = std::time::Instant::now();
        before_each(suite);
        let case_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (case.run)(suite);
        }));
        after_each(suite);
        let elapsed = start.elapsed();
        let ms = elapsed.as_millis();
        if let Err(payload) = case_result {
            if let Some(reason) = payload.downcast_ref::<FailReason>() {
                match reason {
                    FailReason::Fail(msg) => {
                        println!("✗ {} ({}ms)", case.name, ms);
                        eprintln!("suitecase::fail: {}: {}", case.name, msg);
                        failed_names.insert(case.name);
                        if fail_msg.is_none() {
                            fail_msg = Some(msg.clone());
                        }
                    }
                    FailReason::FailNow(msg) => {
                        println!("✗ {} ({}ms)", case.name, ms);
                        eprintln!("suitecase::fail_now: {}: {}", case.name, msg);
                        failed_names.insert(case.name);
                        if fail_now_msg.is_none() {
                            fail_now_msg = Some(msg.clone());
                        }
                        break;
                    }
                }
            } else {
                println!("✗ {} ({}ms)", case.name, ms);
                if let Some(msg) = extract_panic_message(&payload) {
                    eprintln!("suitecase::panic: {}: {}", case.name, msg);
                }
                failed_names.insert(case.name);
                if first_panic.is_none() {
                    first_panic = Some(payload);
                }
            }
        } else {
            println!("✓ {} ({}ms)", case.name, ms);
        }
    }
    teardown_suite(suite);
    if let Some(msg) = fail_now_msg {
        panic!("suitecase: {}", msg);
    }
    if let Some(payload) = first_panic {
        std::panic::resume_unwind(payload);
    }
}

fn extract_panic_message(payload: &Box<dyn std::any::Any + Send>) -> Option<String> {
    if let Some(s) = payload.downcast_ref::<&str>() {
        Some(s.to_string())
    } else if let Some(s) = payload.downcast_ref::<String>() {
        Some(s.clone())
    } else {
        None
    }
}

fn pattern_match(pattern: &str, name: &str) -> bool {
    if is_regex_pattern(pattern) {
        match Regex::new(pattern) {
            Ok(re) => re.is_match(name),
            Err(_) => false,
        }
    } else {
        glob_match(pattern, name)
    }
}

fn is_regex_pattern(pattern: &str) -> bool {
    pattern.chars().any(|c| {
        matches!(
            c,
            '^' | '$' | '.' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '|'
        )
    })
}

fn glob_match(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == name;
    }
    let mut pos = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !name.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else if i == parts.len() - 1 {
            if !name[pos..].ends_with(part) {
                return false;
            }
        } else {
            if let Some(found) = name[pos..].find(part) {
                pos += found + part.len();
            } else {
                return false;
            }
        }
    }
    true
}

fn resolve_deps<'a, S>(
    case: &'a Case<S>,
    all: &std::collections::HashMap<&str, &'a Case<S>>,
    resolved: &mut Vec<&'a Case<S>>,
    seen: &mut std::collections::HashSet<&'a str>,
    visiting: &mut std::collections::HashSet<&'a str>,
) {
    if seen.contains(case.name) {
        return;
    }
    if visiting.contains(case.name) {
        let cycle: Vec<&str> = visiting.iter().copied().collect();
        panic!(
            "suitecase: circular dependency detected: {} → {}",
            case.name,
            cycle.join(" → ")
        );
    }
    visiting.insert(case.name);
    for dep_name in case.dependencies {
        let dep = all.get(dep_name).unwrap_or_else(|| {
            panic!(
                "suitecase: dependency {:?} of case {:?} not found",
                dep_name, case.name
            )
        });
        resolve_deps(dep, all, resolved, seen, visiting);
    }
    visiting.remove(case.name);
    if seen.insert(case.name) {
        resolved.push(case);
    }
}

/// Build a `&'static [Case<S>]` from case names and inline blocks.
///
/// # Declaration
///
/// ```text
/// cases! {
///     $ty:ty, $s:ident => $($name:ident => $body:block),* $(,)?
/// }
/// ```
///
/// With dependencies:
///
/// ```text
/// cases! {
///     $ty:ty, $s:ident =>
///         $name:ident => $body:block,
///         $name:ident (depends_on = [$($dep:ident),+]) => $body:block,
/// }
/// ```
///
/// # Description
///
/// The type and binding `$s` are written once; each `$body` is a block that runs as the case
/// body. Closures must not capture the environment so they coerce to `fn(&mut S)`.
///
/// # Example
///
/// ```
/// use suitecase::{cases, run, Case, HookFns, RunConfig};
///
/// #[derive(Default)]
/// struct MySuite {
///     n: i32,
/// }
///
/// let cases: &[Case<MySuite>] = cases![MySuite, s =>
///     one => { s.n = 1; },
///     two => { assert_eq!(s.n, 1); },
/// ];
/// let mut suite = MySuite::default();
/// run(&mut suite, cases, RunConfig::all(), &HookFns::default());
/// ```
///
/// # Example with dependencies
///
/// ```
/// use suitecase::{cases, run, Case, HookFns, RunConfig};
///
/// #[derive(Default)]
/// struct MySuite {
///     n: i32,
/// }
///
/// let cases: &[Case<MySuite>] = cases![MySuite, s =>
///     setup => { s.n = 1; },
///     verify(depends_on = [setup]) => { assert_eq!(s.n, 1); },
/// ];
/// let mut suite = MySuite::default();
/// run(&mut suite, cases, RunConfig::filter("verify"), &HookFns::default());
/// assert_eq!(suite.n, 1);
/// ```
#[macro_export]
macro_rules! cases {
    ($ty:ty, $s:ident => $($rest:tt)*) => {
        $crate::cases!(@parse ($ty) ($s) [] $($rest)*)
    };
    (@parse ($ty:ty) ($s:ident) [$($out:expr),*] $name:ident (depends_on = [$($dep:ident),+ $(,)?]) => $body:block $(, $($rest:tt)*)?) => {
        $crate::cases!(@parse ($ty) ($s) [$($out,)* $crate::suite::Case::<$ty> { name: stringify!($name), run: |$s: &mut $ty| { let _ = &$s; $body }, dependencies: &[$(stringify!($dep)),+] }] $($($rest)*)?)
    };
    (@parse ($ty:ty) ($s:ident) [$($out:expr),*] $name:ident => $body:block $(, $($rest:tt)*)?) => {
        $crate::cases!(@parse ($ty) ($s) [$($out,)* $crate::suite::Case::<$ty>::new(stringify!($name), |$s: &mut $ty| { let _ = &$s; $body })] $($($rest)*)?)
    };
    (@parse ($ty:ty) ($s:ident) [$($out:expr),*] ,) => {
        &[$($out),*]
    };
    (@parse ($ty:ty) ($s:ident) [$($out:expr),*]) => {
        &[$($out),*]
    };
}

/// Emit a **single** `#[test]` that runs all cases sequentially in slice order, with timing output.
///
/// Each case prints `▶ {name}` before execution and `✓ {name} ({ms}ms)` or
/// `✗ {name} ({ms}ms)` after. Output is machine-parseable for `cargo suitecase test`.
///
/// Use this when cases depend on each other having run first (e.g., setup → mutate → assert).
///
/// # Declaration
///
/// ```text
/// test_suite! {
///     $ty:ty, $storage:ident, $test:ident, $init:expr, $hooks:expr, $s:ident =>
///         $($name:ident => $body:block),* $(,)?
/// }
/// ```
///
/// # Example
///
/// ```
/// use suitecase::{test_suite, HookFns};
///
/// #[derive(Default)]
/// struct Counter { n: i32 }
///
/// test_suite!(
///     Counter,
///     MY_SUITE,
///     counter_test,
///     Counter::default(),
///     HookFns::default(),
///     s =>
///     setup => { s.n = 1; },
///     verify => { assert_eq!(s.n, 1); },
/// );
/// ```
///
/// # Example
///
/// ```
/// use suitecase::{test_suite, HookFns};
///
/// #[derive(Default)]
/// struct Counter { n: i32 }
///
/// test_suite!(
///     Counter,
///     MY_SUITE_2,
///     counter_test_2,
///     Counter::default(),
///     HookFns::default(),
///     s =>
///     setup => { s.n = 1; },
///     verify => { assert_eq!(s.n, 1); },
/// );
/// ```
#[macro_export]
macro_rules! test_suite {
    ($ty:ty, $storage:ident, $test:ident, $init:expr, $hooks:expr, $s:ident =>
        $($name:ident => $body:block),* $(,)?
    ) => {
        static $storage: ::std::sync::OnceLock<::std::sync::Mutex<$ty>> =
            ::std::sync::OnceLock::new();

        #[test]
        fn $test() {
            println!("◆ {} {}", stringify!($storage), stringify!($test));
            let mut suite = $storage
                .get_or_init(|| ::std::sync::Mutex::new($init))
                .lock()
                .expect("suitecase: shared suite mutex poisoned");
            let cases: &[$crate::suite::Case<$ty>] = &[$(
                $crate::suite::Case::<$ty>::new(stringify!($name), |$s: &mut $ty| $body)
            ),*];
            $crate::suite::run(
                &mut *suite,
                cases,
                $crate::suite::RunConfig::all(),
                &$hooks,
            );
        }
    };
}

#[cfg(test)]
mod suite_test;

#[cfg(test)]
mod shared_state_test;

#[cfg(test)]
mod cargo_filter_output_test;

#[cfg(test)]
mod fail_test;

#[cfg(test)]
mod selection_test;
