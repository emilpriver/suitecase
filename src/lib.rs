//! **The structured test toolkit.** A sync Rust library for named **cases**, optional lifecycle
//! hooks, and [`cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html) integration
//! via [`run`]—without a custom harness or DSL.
//!
//! # How a run works
//!
//! 1. You build a static slice of [`Case`] values with [`cases!`] and pass [`HookFns`] plus
//!    [`RunConfig`] to [`run`].
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
//! | [`cases!`](crate::cases) | Build `&'static [Case<S>]` from inline blocks per case |
//! | [`test_suite!`](crate::test_suite) | Emit one `#[test]` per case; all share one `Mutex` suite |
//!
//! # Example
//!
//! ```
//! use suitecase::{cases, run, Case, HookFns, RunConfig};
//!
//! #[derive(Default)]
//! struct Counter {
//!     n: i32,
//! }
//!
//! static CASES: &[Case<Counter>] = cases![Counter, s =>
//!     test_inc => { s.n += 1; },
//! ];
//!
//! let mut suite = Counter::default();
//! run(&mut suite, CASES, RunConfig::all(), &HookFns::default());
//! assert_eq!(suite.n, 1);
//! ```

pub mod suite;

pub use suite::{Case, HookFns, RunConfig, run};
