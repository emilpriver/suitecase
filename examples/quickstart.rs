//! Mirrors the README **Quickstart** section.
//!
//! Run: `cargo test --example quickstart`
//!
//! One `#[test]` runs every case in order on **one** suite via [`RunConfig::all`]. For **one line
//! per case in `cargo test`** with a shared suite, use [`test_suite!`] in the crate docs.

#![allow(dead_code)]

use suitecase::{cases, Case, HookFns, RunConfig, run};

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
            self.n,
            1,
            "test_inc should have run first on the same suite"
        );
    }
}

static MY_CASES: &[Case<Counter>] = cases![Counter, s =>
    test_inc => { s.test_inc(); },
    test_inc_verify => { s.test_inc_verify(); },
];

static MY_HOOKS: HookFns<Counter> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

fn quickstart_body() {
    let mut suite = Counter::default();
    run(&mut suite, MY_CASES, RunConfig::all(), &MY_HOOKS);
}

#[test]
fn quickstart() {
    quickstart_body();
}

fn main() {
    quickstart_body();
}
