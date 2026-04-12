//! **The structured test toolkit.** A sync Rust library for named **cases**, optional lifecycle
//! hooks, and [`cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html) integration
//! via [`run`]—without a custom harness or DSL.
//!
//! # How a run works
//!
//! 1. You build a static slice of [`Case`] values (via [`suite_methods!`], [`cases!`], or
//!    [`cases_fn!`]) and pass [`HookFns`] plus [`RunConfig`] to [`run`].
//! 2. [`run`] **selects** which cases to execute: either all of them, or only the one whose
//!    [`Case::name`] equals [`RunConfig::filter`].
//! 3. If nothing is selected and a filter was set, it **panics** (likely a typo). If nothing is
//!    selected and there is **no** filter, it returns immediately (empty case list).
//! 4. Otherwise it runs optional hooks in order: `setup_suite` → for each selected case:
//!    `before_each` → case body → `after_each` → then `teardown_suite`. Hook slots that are
//!    [`None`] in [`HookFns`] are skipped.
//!
//! Panics inside a case are caught so `after_each` still runs; the panic is re-raised afterward.
//!
//! # Macros
//!
//! | Macro | Role |
//! |-------|------|
//! | [`suite_methods!`](crate::suite_methods) | Cases from inherent `test_*` methods |
//! | [`cases!`](crate::cases) | Cases from inline blocks |
//! | [`cases_fn!`](crate::cases_fn) | Cases from `fn(&mut S)` pointers |
//! | [`cargo_case_tests!`](crate::cargo_case_tests) | Emit one `#[test]` per case (default hooks) |
//! | [`cargo_case_tests_with_hooks!`](crate::cargo_case_tests_with_hooks) | Same, with your [`HookFns`] |
//!
//! # Example
//!
//! ```
//! use suitecase::{run, suite_methods, Case, HookFns, RunConfig};
//!
//! #[derive(Default)]
//! struct Counter {
//!     n: i32,
//! }
//!
//! impl Counter {
//!     fn test_inc(&mut self) {
//!         self.n += 1;
//!     }
//! }
//!
//! static CASES: &[Case<Counter>] = suite_methods![Counter, s => test_inc];
//!
//! let mut suite = Counter::default();
//! run(&mut suite, CASES, RunConfig::all(), &HookFns::default());
//! assert_eq!(suite.n, 1);
//! ```

pub mod suite;

pub use suite::{Case, HookFns, RunConfig, run};
