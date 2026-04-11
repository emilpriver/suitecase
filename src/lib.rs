//! Test suites inspired by [Go's testify/suite](https://pkg.go.dev/github.com/stretchr/testify/suite):
//! shared **setup** / **cleanup** (suite level) and **before_each** / **after_each** (per case).
//!
//! ## testify/suite mapping
//!
//! | testify (`suite` package) | suitcase |
//! |---------------------------|----------|
//! | `SetupSuite`              | [`Suite::setup_suite`] |
//! | `TearDownSuite`           | [`Suite::teardown_suite`] |
//! | `SetupTest`               | [`Suite::before_each`] |
//! | `TearDownTest`            | [`Suite::after_each`] |
//!
//! In testify, `suite.Run` collects `Test*` methods, runs `SetupSuite` once, then for each test:
//! `SetupTest` → test → `TearDownTest`, and finally `TearDownSuite`. If **no** test methods match
//! the filter, testify does **not** call `SetupSuite` — [`run`] does the same.
//!
//! ## `Suite` and `test_*` methods
//!
//! Implement [`Suite`] for your type with **all four** hooks (there are no default empty
//! implementations). Define each case as an **inherent** method with the `test_` prefix, then list
//! those methods in [`suite_methods!`] to build the `&[Case<Self>]` passed to [`run`]:
//!
//! ```rust
//! use suitcase::{run, suite_methods, Case, RunConfig, Suite};
//!
//! #[derive(Default)]
//! struct MySuite {
//!     i: i32,
//! }
//!
//! impl MySuite {
//!     fn test_one(&mut self) {
//!         self.i = 1;
//!     }
//!     fn test_two(&mut self) {
//!         assert_eq!(self.i, 1);
//!     }
//! }
//!
//! impl Suite for MySuite {
//!     fn setup_suite(&mut self) {}
//!     fn teardown_suite(&mut self) {}
//!     fn before_each(&mut self) {}
//!     fn after_each(&mut self) {}
//! }
//!
//! static CASES: &[Case<MySuite>] = suite_methods![MySuite, s => test_one, test_two];
//!
//! fn main() {
//!     let mut suite = MySuite::default();
//!     run(&mut suite, CASES, RunConfig::all());
//! }
//! ```
//!
//! Case names in [`RunConfig::filter`] match the method name (e.g. `"test_one"`).
//!
//! You can still use [`cases!`] or [`cases_fn!`] if you prefer not to use inherent `test_*`
//! methods.
//!
//! ## Running one case (singular test) with full hooks
//!
//! Use [`RunConfig`] with a filter so only one case runs. [`Suite::setup_suite`] and
//! [`Suite::teardown_suite`] still run around that single case (when at least one case matches).
//!
//! - **Programmatic:** `run(suite, cases, RunConfig::filter("test_only_this"))`
//! - **CLI / env:** `SUITCASE_FILTER=test_only_this cargo test my_suite_test`
//!
//! ## Macros
//!
//! - **[`suite_methods!`]** — Build `&[Case<S>]` from inherent `test_*` method names (recommended).
//! - **[`cases!`]** — Inline case bodies (names need not be `test_*`).
//! - **[`cases_fn!`]** — Plain `fn(&mut S)` pointers.
//! - **[`cargo_case_tests!`]** — One `#[test]` per case name for `cargo test` output.
//!
//! ## `cargo test` and filtering
//!
//! Rust names individual tests by `#[test]` function. To filter by **case name** inside one
//! `#[test]`, use [`RunConfig::from_env`] and `SUITCASE_FILTER`, or pass [`RunConfig::filter`]
//! yourself.

/// Suite lifecycle hooks. **All four methods must be implemented** (there are no defaults).
///
/// Test bodies are inherent methods named `test_*` on your suite type; wire them with
/// [`suite_methods!`] and [`run`].
pub trait Suite {
    /// Runs once before any case in this run, after the filter is applied and the set of cases
    /// is non-empty (same idea as testify skipping `SetupSuite` when nothing runs).
    fn setup_suite(&mut self);

    /// Runs once after all selected cases finish (including after the last `after_each`).
    fn teardown_suite(&mut self);

    /// Runs before each selected case.
    fn before_each(&mut self);

    /// Runs after each selected case, even if the case panicked.
    fn after_each(&mut self);
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

/// Runs `cases` on `suite` with hook ordering aligned to testify/suite:
///
/// 1. Determine the selected cases from `config.filter` (exact name match).
/// 2. If that set is **empty**, return immediately **without** calling [`Suite::setup_suite`].
/// 3. Otherwise: [`Suite::setup_suite`], then for each selected case:
///    [`Suite::before_each`] → case body → [`Suite::after_each`] (after_each runs even if the
///    case panics), then [`Suite::teardown_suite`] once at the end.
///
/// Panics in a case are propagated after `after_each` runs.
pub fn run<S: Suite>(suite: &mut S, cases: &[Case<S>], config: RunConfig) {
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

    suite.setup_suite();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for case in selected {
            suite.before_each();
            let case_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                (case.run)(suite);
            }));
            suite.after_each();
            if let Err(payload) = case_result {
                std::panic::resume_unwind(payload);
            }
        }
    }));
    suite.teardown_suite();
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
/// use suitcase::{suite_methods, Case, Suite};
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
/// impl Suite for MySuite {
///     fn setup_suite(&mut self) {}
///     fn teardown_suite(&mut self) {}
///     fn before_each(&mut self) {}
///     fn after_each(&mut self) {}
/// }
///
/// let _cases = suite_methods![MySuite, s => test_a];
/// ```
#[macro_export]
macro_rules! suite_methods {
    ($ty:ty, $s:ident => $($name:ident),* $(,)?) => {
        &[$( $crate::Case::<$ty>::new(stringify!($name), |$s: &mut $ty| { $s.$name(); })),*]
    };
}

/// Build a `&'static [Case<S>]` from case names and inline blocks.
///
/// The suite type and parameter name are written once: `cases![MySuite, s => …]`. Use that
/// identifier in each `=> { … }` body. Closures coerce to function pointers when they capture
/// nothing from the environment.
///
/// ```
/// use suitcase::{cases, Case, Suite};
///
/// #[derive(Default)]
/// struct MySuite {
///     field: i32,
/// }
///
/// impl Suite for MySuite {
///     fn setup_suite(&mut self) {}
///     fn teardown_suite(&mut self) {}
///     fn before_each(&mut self) {}
///     fn after_each(&mut self) {}
/// }
///
/// let _cases = cases![MySuite, s =>
///     first => { s.field = 1; },
///     second => { assert_eq!(s.field, 1); },
/// ];
/// ```
#[macro_export]
macro_rules! cases {
    ($ty:ty, $s:ident => $($name:ident => $body:block),* $(,)?) => {
        &[$( $crate::Case::<$ty>::new(stringify!($name), |$s: &mut $ty| $body)),*]
    };
}

/// Build a `&'static [Case<S>]` from case names and plain function pointers (no captures).
///
/// ```
/// use suitcase::{cases_fn, Case, Suite};
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
/// impl Suite for MySuite {
///     fn setup_suite(&mut self) {}
///     fn teardown_suite(&mut self) {}
///     fn before_each(&mut self) {}
///     fn after_each(&mut self) {}
/// }
///
/// let _cases = cases_fn![MySuite => first => first, second => second];
/// ```
#[macro_export]
macro_rules! cases_fn {
    ($ty:ty => $($name:ident => $fn:path),* $(,)?) => {
        &[$( $crate::Case::<$ty>::new(stringify!($name), $fn as fn(&mut $ty))),*]
    };
}

/// Emit one `#[test]` per case name, each calling [`run`] with [`RunConfig::filter`] set to that
/// case’s name (so it shows up separately in `cargo test` output).
///
/// Pass the same `cases` slice you use for a full [`run`] with [`RunConfig::all`]. Each case body
/// must behave correctly when it is the **only** case selected (after [`Suite::setup_suite`] /
/// [`Suite::before_each`]). If a case depends on earlier mutations, encode that in the body (for
/// example by branching on `s.value`) or keep a separate sequential `#[test]` that runs all cases.
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
                $crate::run(&mut suite, $cases, $crate::RunConfig::filter(stringify!($name)));
            }
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Recorder {
        log: Vec<&'static str>,
    }

    impl Recorder {
        fn push(&mut self, e: &'static str) {
            self.log.push(e);
        }
    }

    impl Recorder {
        fn test_a(&mut self) {
            self.push("a");
        }

        fn test_b(&mut self) {
            self.push("b");
        }

        fn test_only(&mut self) {
            self.push("only");
        }

        fn test_bad(&mut self) {
            let _ = self;
            panic!("boom");
        }
    }

    impl Suite for Recorder {
        fn setup_suite(&mut self) {
            self.push("setup_suite");
        }

        fn teardown_suite(&mut self) {
            self.push("teardown_suite");
        }

        fn before_each(&mut self) {
            self.push("before_each");
        }

        fn after_each(&mut self) {
            self.push("after_each");
        }
    }

    #[test]
    fn hook_order_all_cases() {
        let mut suite = Recorder::default();
        run(
            &mut suite,
            suite_methods![Recorder, s => test_a, test_b],
            RunConfig::all(),
        );

        assert_eq!(
            suite.log,
            vec![
                "setup_suite",
                "before_each",
                "a",
                "after_each",
                "before_each",
                "b",
                "after_each",
                "teardown_suite",
            ]
        );
    }

    #[test]
    fn singular_filtered_case_runs_setup_and_teardown() {
        let mut suite = Recorder::default();
        run(
            &mut suite,
            suite_methods![Recorder, s => test_a, test_b],
            RunConfig::filter("test_b"),
        );

        assert_eq!(
            suite.log,
            vec![
                "setup_suite",
                "before_each",
                "b",
                "after_each",
                "teardown_suite",
            ]
        );
    }

    #[test]
    fn cases_fn_macro_works() {
        fn a(s: &mut Recorder) {
            s.push("a");
        }
        fn b(s: &mut Recorder) {
            s.push("b");
        }
        let mut suite = Recorder::default();
        run(
            &mut suite,
            cases_fn![Recorder => a => a, b => b],
            RunConfig::all(),
        );
        assert_eq!(
            suite.log,
            vec![
                "setup_suite",
                "before_each",
                "a",
                "after_each",
                "before_each",
                "b",
                "after_each",
                "teardown_suite",
            ]
        );
    }

    #[test]
    fn no_cases_no_setup() {
        let mut suite = Recorder::default();
        run(&mut suite, &[], RunConfig::all());
        assert!(suite.log.is_empty());
    }

    #[test]
    #[should_panic(expected = "matched no cases")]
    fn invalid_filter_panics() {
        let mut suite = Recorder::default();
        run(
            &mut suite,
            suite_methods![Recorder, s => test_only],
            RunConfig::filter("nope"),
        );
    }

    #[test]
    fn after_each_runs_when_case_panics() {
        let mut suite = Recorder::default();
        let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run(
                &mut suite,
                suite_methods![Recorder, s => test_bad],
                RunConfig::all(),
            );
        }));
        assert!(err.is_err());
        assert_eq!(
            suite.log,
            vec!["setup_suite", "before_each", "after_each", "teardown_suite",]
        );
    }
}
