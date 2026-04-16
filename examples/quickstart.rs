//! Mirrors the README **Quickstart** section.
//!
//! Run: `cargo test --example quickstart` — two `#[test]` lines from [`test_suite!`], sharing one
//! suite and running in registration order. [`main`] uses [`RunConfig::all`] so
//! `cargo run --example quickstart` runs every case in order on one suite.

#![allow(dead_code)]

use suitecase::{HookFns, RunConfig, run, test_suite};

#[derive(Default)]
struct Counter {
    n: i32,
}

impl Counter {
    fn test_inc(&mut self) {
        self.n += 1;
    }

    fn test_inc_verify(&mut self) {
        assert_eq!(
            self.n, 1,
            "test_inc should have run first on the same suite"
        );
    }
}

static MY_HOOKS: HookFns<Counter> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

test_suite!(
    Counter,
    MY_SHARED_SUITE,
    MY_CURSOR,
    MY_CASES,
    Counter::default(),
    MY_HOOKS,
    s =>
    test_inc => { s.test_inc(); },
    test_inc_verify => { s.test_inc_verify(); },
);

fn quickstart_body() {
    let mut suite = Counter::default();
    run(&mut suite, MY_CASES, RunConfig::all(), &MY_HOOKS);
}

fn main() {
    quickstart_body();
}
