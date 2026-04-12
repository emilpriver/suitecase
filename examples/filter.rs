//! Mirrors the README **Run a single case (filter)** section — each case name is its own test
//! (same idea as [`RunConfig::filter`] per run).
//!
//! Run: `cargo test --example filter`

#![allow(dead_code)]

use suitecase::{cargo_case_tests_with_hooks, suite_methods, Case, HookFns};

#[derive(Default)]
struct Counter {
    n: i32,
}

impl Counter {
    fn test_a(&mut self) {
        self.n = 1;
    }
    fn test_b(&mut self) {
        self.n = 2;
    }
}

static MY_CASES: &[Case<Counter>] = suite_methods![Counter, s => test_a, test_b];

static MY_HOOKS: HookFns<Counter> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

cargo_case_tests_with_hooks!(Counter::default(), MY_CASES, MY_HOOKS, [test_a, test_b]);

fn main() {}
