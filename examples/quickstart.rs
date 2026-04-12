//! Mirrors the README **Quickstart** section.
//!
//! Run: `cargo test --example quickstart`

#![allow(dead_code)]

use suitecase::{cargo_case_tests_with_hooks, suite_methods, Case, HookFns};

#[derive(Default)]
struct Counter {
    n: i32,
}

impl Counter {
    fn test_inc(&mut self) {
        self.n += 1;
    }
}

static MY_CASES: &[Case<Counter>] = suite_methods![Counter, s => test_inc];

static MY_HOOKS: HookFns<Counter> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

cargo_case_tests_with_hooks!(Counter::default(), MY_CASES, MY_HOOKS, [test_inc]);

fn main() {}
