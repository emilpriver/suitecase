//! Mirrors the README **Hooks** section.
//!
//! Run: `cargo test --example hooks`

#![allow(dead_code)]

use suitecase::{cargo_case_tests_with_hooks, suite_methods, Case, HookFns};

#[derive(Default)]
struct State {
    log: Vec<&'static str>,
}

impl State {
    fn test_ok(&mut self) {
        self.log.push("case");
    }
}

fn setup(s: &mut State) {
    s.log.push("setup");
}

static MY_CASES: &[Case<State>] = suite_methods![State, s => test_ok];

static MY_HOOKS: HookFns<State> = HookFns {
    setup_suite: Some(setup),
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

cargo_case_tests_with_hooks!(State::default(), MY_CASES, MY_HOOKS, [test_ok]);

fn main() {}
