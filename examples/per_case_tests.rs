//! Mirrors the README **Show each case in cargo test** section.
//!
//! The macro emits one `#[test]` per listed name. Run:
//! `cargo test --example per_case_tests`

#![allow(dead_code)]

use suitecase::{cargo_case_tests_with_hooks, suite_methods, Case, HookFns};

#[derive(Default)]
struct MySuite {
    n: i32,
}

impl MySuite {
    fn test_a(&mut self) {
        self.n = 1;
    }
    fn test_b(&mut self) {
        self.n = 2;
    }
}

static MY_CASES: &[Case<MySuite>] = suite_methods![MySuite, s => test_a, test_b];

fn setup(s: &mut MySuite) {
    s.n = 0;
}

static MY_HOOKS: HookFns<MySuite> = HookFns {
    setup_suite: Some(setup),
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

cargo_case_tests_with_hooks!(MySuite::default(), MY_CASES, MY_HOOKS, [test_a, test_b]);

fn main() {}
