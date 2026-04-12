//! Test suites with shared **setup** / **teardown** (suite level) and **before_each** / **after_each**
//! (per case).
//!
//! The suite runner and related types live in [`suite`]. Re-exports at the crate root match the
//! historical `use suitcase::{…}` paths.
//!
//! ## Lifecycle hooks
//!
//! | Role | [`HookFns`](crate::suite::HookFns) field |
//! |------|----------------------------------------|
//! | Once before any selected case | [`setup_suite`](crate::suite::HookFns::setup_suite) |
//! | Once after all selected cases | [`teardown_suite`](crate::suite::HookFns::teardown_suite) |
//! | Before each case | [`before_each`](crate::suite::HookFns::before_each) |
//! | After each case | [`after_each`](crate::suite::HookFns::after_each) |
//!
//! [`run`](crate::suite::run) applies a filter first; if **no** cases match, it returns without
//! calling any hooks. Otherwise it runs `setup_suite`, then for each case `before_each` → case →
//! `after_each`, then `teardown_suite`.
//!
//! ## Hooks and `test_*` methods
//!
//! Pass a [`HookFns`](crate::suite::HookFns) to [`run`](crate::suite::run): each field is
//! `Option<fn(&mut S)>`. Use [`Some`] with a function for that hook, or [`None`] to skip it.
//! [`HookFns::default`](crate::suite::HookFns::default) is all [`None`] (only case bodies run).
//!
//! Define each case as an **inherent** method with the `test_` prefix, then list those methods in
//! [`suite_methods!`] to build the `&[Case<Self>]` passed to [`run`]:
//!
//! ```rust
//! use suitcase::{run, suite_methods, Case, HookFns, RunConfig};
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
//! static CASES: &[Case<MySuite>] = suite_methods![MySuite, s => test_one, test_two];
//!
//! fn main() {
//!     let mut suite = MySuite::default();
//!     run(&mut suite, CASES, RunConfig::all(), &HookFns::default());
//! }
//! ```
//!
//! Case names in [`RunConfig::filter`](crate::suite::RunConfig::filter) match the method name (e.g. `"test_one"`).
//!
//! You can still use [`cases!`] or [`cases_fn!`] if you prefer not to use inherent `test_*`
//! methods.
//!
//! ## Running one case (singular test) with full hooks
//!
//! Use [`RunConfig`](crate::suite::RunConfig) with a filter so only one case runs. Any [`Some`] hooks
//! in [`HookFns`] still run around that single case (when at least one case matches).
//!
//! - **Programmatic:** `run(suite, cases, RunConfig::filter("test_only_this"), &hooks)`
//! - **CLI / env:** `SUITCASE_FILTER=test_only_this cargo test my_suite_test`
//!
//! ## Macros
//!
//! - **[`suite_methods!`]** — Build `&[Case<S>]` from inherent `test_*` method names (recommended).
//! - **[`cases!`]** — Inline case bodies (names need not be `test_*`).
//! - **[`cases_fn!`]** — Plain `fn(&mut S)` pointers.
//! - **[`cargo_case_tests!`]** — One `#[test]` per case name for `cargo test` output (uses default [`HookFns`]).
//! - **[`cargo_case_tests_with_hooks!`]** — Same with your [`HookFns`].
//!
//! ## `cargo test` and filtering
//!
//! Rust names individual tests by `#[test]` function. To filter by **case name** inside one
//! `#[test]`, use [`RunConfig::from_env`](crate::suite::RunConfig::from_env) and `SUITCASE_FILTER`, or pass [`RunConfig::filter`](crate::suite::RunConfig::filter)
//! yourself.

pub mod suite;

pub use suite::{Case, HookFns, RunConfig, run};
