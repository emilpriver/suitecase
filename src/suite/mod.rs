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
}

impl<S> Case<S> {
    pub const fn new(name: &'static str, run: fn(&mut S)) -> Self {
        Self { name, run }
    }
}

/// Controls **which** cases [`run`] executes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunConfig {
    /// When set, only the case whose name matches **exactly** runs.
    pub filter: Option<String>,
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
        }
    }
}

/// Run the selected cases on `suite` using `hooks` for lifecycle callbacks.
///
/// See the [module-level **Execution model** section](crate::suite#execution-model) for ordering and
/// panic behavior.
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
#[macro_export]
macro_rules! cases {
    ($ty:ty, $s:ident => $($name:ident => $body:block),* $(,)?) => {
        &[$( $crate::suite::Case::<$ty>::new(stringify!($name), |$s: &mut $ty| $body)),*]
    };
}

/// Emit one `#[test]` per listed case name. Each test locks a **shared** suite behind a
/// [`std::sync::Mutex`] in a [`std::sync::OnceLock`] keyed by `$storage`, then calls [`run`] with
/// [`RunConfig::filter`] and your [`HookFns`].
///
/// `$ty` must be [`Send`]. The harness may run tests in any order or in parallel; if one case
/// depends on another having run first on the same suite, see the macro docs on ordering, or call
/// [`run`] once with [`RunConfig::all`] instead.
///
/// # Declaration
///
/// **Split cases and names** — pass a [`cases!`] slice (or any `&[Case<S>]`) plus a matching list of
/// identifiers for the generated `#[test]` functions:
///
/// ```text
/// test_suite! {
///     $ty:ty, $storage:ident, $init:expr, $cases:expr, $hooks:expr, [$($name:ident),* $(,)?] $(,)?
/// }
/// ```
///
/// **Inline cases** — same syntax as [`cases!`], written once; the macro defines `$cases_static`
/// and emits one `#[test] fn $name` per case:
///
/// ```text
/// test_suite! {
///     $ty:ty, $storage:ident, $cases_static:ident, $init:expr, $hooks:expr, $s:ident =>
///         $($name:ident => $body:block),* $(,)?
/// }
/// ```
#[macro_export]
macro_rules! test_suite {
    ($ty:ty, $storage:ident, $init:expr, $cases:expr, $hooks:expr, [$($name:ident),* $(,)?] $(,)?) => {
        static $storage: ::std::sync::OnceLock<::std::sync::Mutex<$ty>> =
            ::std::sync::OnceLock::new();

        $(
            #[test]
            fn $name() {
                let mut suite = $storage
                    .get_or_init(|| ::std::sync::Mutex::new($init))
                    .lock()
                    .expect("suitecase: shared suite mutex poisoned");
                $crate::suite::run(
                    &mut *suite,
                    $cases,
                    $crate::suite::RunConfig::filter(stringify!($name)),
                    &$hooks,
                );
            }
        )*
    };

    ($ty:ty, $storage:ident, $cases_static:ident, $init:expr, $hooks:expr, $s:ident =>
        $($name:ident => $body:block),* $(,)?
    ) => {
        static $cases_static: &'static [$crate::suite::Case<$ty>] = &[$(
            $crate::suite::Case::<$ty>::new(stringify!($name), |$s: &mut $ty| $body)
        ),*];

        static $storage: ::std::sync::OnceLock<::std::sync::Mutex<$ty>> =
            ::std::sync::OnceLock::new();

        $(
            #[test]
            fn $name() {
                let mut suite = $storage
                    .get_or_init(|| ::std::sync::Mutex::new($init))
                    .lock()
                    .expect("suitecase: shared suite mutex poisoned");
                $crate::suite::run(
                    &mut *suite,
                    $cases_static,
                    $crate::suite::RunConfig::filter(stringify!($name)),
                    &$hooks,
                );
            }
        )*
    };
}

#[cfg(test)]
mod suite_test;

#[cfg(test)]
mod shared_state_test;

#[cfg(test)]
mod cargo_filter_output_test;
