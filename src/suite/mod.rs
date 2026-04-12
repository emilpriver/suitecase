//! Suite runner: lifecycle hooks, case lists, and [`run`].
//!
//! Add test-side helpers alongside this module as the library grows.

/// Hooks as optional function pointers. Pass [`Some`] with a `fn(&mut S)` to run a hook, or
/// [`None`] to skip it (no-op for that slot).
///
/// [`Default`] is all [`None`] — nothing runs except your case bodies.
#[derive(Clone, Copy, Debug)]
pub struct HookFns<S> {
    pub setup_suite: Option<fn(&mut S)>,
    pub teardown_suite: Option<fn(&mut S)>,
    pub before_each: Option<fn(&mut S)>,
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
    /// Alias for [`Default::default`] — every hook is [`None`].
    pub fn noop() -> Self {
        Self::default()
    }
}

/// One named case in a suite. Holds a plain function pointer so cases can live in `static`
/// slices (non-capturing closures coerce to `fn(&mut S)`).
pub struct Case<S> {
    pub name: &'static str,
    pub run: fn(&mut S),
}

impl<S> Case<S> {
    pub const fn new(name: &'static str, run: fn(&mut S)) -> Self {
        Self { name, run }
    }
}

/// Configuration for [`run`], mainly case filtering (singular / subset runs).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunConfig {
    /// When `Some`, only cases whose [`Case::name`] matches **exactly** are run.
    pub filter: Option<String>,
}

impl RunConfig {
    /// Run every case in the slice (subject to the “no matching cases” rule in [`run`]).
    pub fn all() -> Self {
        Self::default()
    }

    /// Run a single case (or any case whose name equals `name`).
    pub fn filter(name: impl Into<String>) -> Self {
        Self {
            filter: Some(name.into()),
        }
    }

    /// Reads [`RunConfig::filter`] from the environment variable `SUITCASE_FILTER`.
    /// Empty or unset means no filter (run all cases).
    pub fn from_env() -> Self {
        Self {
            filter: std::env::var("SUITCASE_FILTER")
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }
}

/// Runs `cases` on `suite` with this hook order:
///
/// 1. Determine the selected cases from `config.filter` (exact name match).
/// 2. If that set is **empty**, return immediately **without** calling any hook.
/// 3. Otherwise: `hooks.setup_suite` if [`Some`], then for each selected case:
///    `hooks.before_each` if [`Some`] → case body → `hooks.after_each` if [`Some`]
///    (`after_each` runs even if the case panics), then `hooks.teardown_suite` if [`Some`] once at the end.
///
/// Panics in a case are propagated after `after_each` runs.
pub fn run<S>(suite: &mut S, cases: &[Case<S>], config: RunConfig, hooks: &HookFns<S>) {
    run_hooks(
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

fn run_hooks<S, FS, FT, FB, FA>(
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
    let selected: Vec<&Case<S>> = cases
        .iter()
        .filter(|c| match &config.filter {
            None => true,
            Some(f) => c.name == f,
        })
        .collect();

    if selected.is_empty() {
        assert!(
            config.filter.is_none(),
            "suitcase: filter {:?} matched no cases",
            config.filter
        );
        return;
    }

    setup_suite(suite);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for case in selected {
            before_each(suite);
            let case_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                (case.run)(suite);
            }));
            after_each(suite);
            if let Err(payload) = case_result {
                std::panic::resume_unwind(payload);
            }
        }
    }));
    teardown_suite(suite);
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

/// Build `&'static [Case<S>]` from inherent **`test_*`** methods on `S`.
///
/// Each identifier must name a method `fn test_…(&mut self)` on `S`. The case name is the full
/// method name (e.g. `"test_foo"`).
///
/// ```
/// use suitcase::{suite_methods, Case, HookFns, RunConfig, run};
///
/// struct MySuite {
///     x: i32,
/// }
///
/// impl MySuite {
///     fn test_a(&mut self) {
///         self.x = 1;
///     }
/// }
///
/// let _cases = suite_methods![MySuite, s => test_a];
/// # let mut suite = MySuite { x: 0 };
/// # run(&mut suite, _cases, RunConfig::all(), &HookFns::default());
/// ```
#[macro_export]
macro_rules! suite_methods {
    ($ty:ty, $s:ident => $($name:ident),* $(,)?) => {
        &[$( $crate::suite::Case::<$ty>::new(stringify!($name), |$s: &mut $ty| { $s.$name(); })),*]
    };
}

/// Build a `&'static [Case<S>]` from case names and inline blocks.
///
/// The suite type and parameter name are written once: `cases![MySuite, s => …]`. Use that
/// identifier in each `=> { … }` body. Closures coerce to function pointers when they capture
/// nothing from the environment.
///
/// ```
/// use suitcase::{cases, Case, HookFns, RunConfig, run};
///
/// #[derive(Default)]
/// struct MySuite {
///     field: i32,
/// }
///
/// let _cases = cases![MySuite, s =>
///     first => { s.field = 1; },
///     second => { assert_eq!(s.field, 1); },
/// ];
/// # let mut suite = MySuite::default();
/// # run(&mut suite, _cases, RunConfig::all(), &HookFns::default());
/// ```
#[macro_export]
macro_rules! cases {
    ($ty:ty, $s:ident => $($name:ident => $body:block),* $(,)?) => {
        &[$( $crate::suite::Case::<$ty>::new(stringify!($name), |$s: &mut $ty| $body)),*]
    };
}

/// Build a `&'static [Case<S>]` from case names and plain function pointers (no captures).
///
/// ```
/// use suitcase::{cases_fn, Case, HookFns, RunConfig, run};
///
/// #[derive(Default)]
/// struct MySuite {
///     field: i32,
/// }
///
/// fn first(s: &mut MySuite) {
///     s.field = 1;
/// }
/// fn second(s: &mut MySuite) {
///     assert_eq!(s.field, 1);
/// }
///
/// let _cases = cases_fn![MySuite => first => first, second => second];
/// # let mut suite = MySuite::default();
/// # run(&mut suite, _cases, RunConfig::all(), &HookFns::default());
/// ```
#[macro_export]
macro_rules! cases_fn {
    ($ty:ty => $($name:ident => $fn:path),* $(,)?) => {
        &[$( $crate::suite::Case::<$ty>::new(stringify!($name), $fn as fn(&mut $ty))),*]
    };
}

/// Emit one `#[test]` per case name, each calling [`run`] with [`RunConfig::filter`] set to that
/// case’s name (so it shows up separately in `cargo test` output). Uses [`HookFns::default`]
/// (no hooks).
///
/// Pass the same `cases` slice you use for a full [`run`] with [`RunConfig::all`]. Each case body
/// must behave correctly when it is the **only** case selected (after any `setup_suite` /
/// `before_each` you pass in other tests). If a case depends on earlier mutations, encode that in
/// the body (for example by branching on `s.value`) or keep a separate sequential `#[test]` that
/// runs all cases.
///
/// This macro expands to several `#[test]` functions at the **crate root** of the test module, so
/// it cannot be demonstrated as a single `rustdoc` snippet. Use it in `tests/*.rs` next to your
/// suite (see the `tests/basic.rs` integration test in this repository).
///
/// ```text
/// static MY_CASES: &[Case<MySuite>] = suite_methods![MySuite, s => test_a, test_b];
///
/// cargo_case_tests!(MySuite::default(), MY_CASES, [test_a, test_b]);
/// ```
#[macro_export]
macro_rules! cargo_case_tests {
    ($suite:expr, $cases:expr, [$($name:ident),* $(,)?] $(,)?) => {
        $(
            #[test]
            fn $name() {
                let mut suite = $suite;
                $crate::suite::run(
                    &mut suite,
                    $cases,
                    $crate::suite::RunConfig::filter(stringify!($name)),
                    &$crate::suite::HookFns::default(),
                );
            }
        )*
    };
}

/// Like [`cargo_case_tests!`], but passes a shared [`HookFns`] (e.g. with [`Some`] hook functions).
#[macro_export]
macro_rules! cargo_case_tests_with_hooks {
    ($suite:expr, $cases:expr, $hooks:expr, [$($name:ident),* $(,)?] $(,)?) => {
        $(
            #[test]
            fn $name() {
                let mut suite = $suite;
                $crate::suite::run(
                    &mut suite,
                    $cases,
                    $crate::suite::RunConfig::filter(stringify!($name)),
                    &$hooks,
                );
            }
        )*
    };
}
