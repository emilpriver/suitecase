//! Example with a failing test case, used to verify failure output formatting.
//!
//! Run: `cargo test --example failing` — one `#[test]` from [`test_suite!`],
//! running two cases where the second intentionally fails.
//! `cargo run --example failing` also runs the cases directly.

#![allow(dead_code)]

use suitecase::{Case, HookFns, RunConfig, cases, run, test_suite};

#[derive(Default)]
struct Counter {
    n: i32,
}

static FAILING_CASES: &[Case<Counter>] = cases![Counter, s =>
    test_pass => { s.n += 1; },
    test_fail => { assert_eq!(s.n, 999, "expected 999 but got {}", s.n); },
];

test_suite!(
    Counter,
    FAILING_SUITE,
    failing_test_run,
    Counter::default(),
    HookFns::default(),
    s =>
    test_pass => { s.n += 1; },
    test_fail => { assert_eq!(s.n, 999, "expected 999 but got {}", s.n); },
);

fn failing_body() {
    let mut suite = Counter::default();
    run(&mut suite, FAILING_CASES, RunConfig::all(), &HookFns::default());
}

fn main() {
    failing_body();
}
