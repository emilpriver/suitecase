//! Mirrors the README **Quickstart** section.
//!
//! Run: `cargo test --example quickstart` — one `#[test]` from [`test_suite!`],
//! running all cases in order on a shared suite. [`main`] uses [`RunConfig::all`] so
//! `cargo run --example quickstart` runs every case in order on one suite.

#![allow(dead_code)]

use suitecase::{Case, HookFns, RunConfig, cases, run, test_suite};

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

    fn test_double(&mut self) {
        self.n *= 2;
    }

    fn test_add(&mut self, val: i32) {
        self.n += val;
    }

    fn test_sub(&mut self, val: i32) {
        self.n -= val;
    }

    fn test_mul(&mut self, val: i32) {
        self.n *= val;
    }

    fn test_div(&mut self, val: i32) {
        self.n /= val;
    }

    fn test_mod(&mut self, val: i32) {
        self.n %= val;
    }

    fn test_negate(&mut self) {
        self.n = -self.n;
    }

    fn test_square(&mut self) {
        self.n = self.n * self.n;
    }

    fn test_cube(&mut self) {
        self.n = self.n * self.n * self.n;
    }

    fn test_increment_by_ten(&mut self) {
        self.n += 10;
    }
}

static MY_HOOKS: HookFns<Counter> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

static MY_CASES: &[Case<Counter>] = cases![Counter, s =>
    test_inc => { s.test_inc(); },
    test_inc_verify => { s.test_inc_verify(); },
    test_double => { s.test_double(); },
    test_add_five => { s.test_add(5); },
    test_sub_three => { s.test_sub(3); },
    test_mul_by_four => { s.test_mul(4); },
    test_div_by_two => { s.test_div(2); },
    test_mod_three => { s.test_mod(3); },
    test_negate => { s.test_negate(); },
    test_square => { s.test_square(); },
    test_cube => { s.test_cube(); },
    test_increment_by_ten => { s.test_increment_by_ten(); },
];

test_suite!(
    Counter,
    MY_SHARED_SUITE,
    quickstart_test_run,
    Counter::default(),
    MY_HOOKS,
    s =>
    test_inc => { s.test_inc(); },
    test_inc_verify => { s.test_inc_verify(); },
    test_double => { s.test_double(); },
    test_add_five => { s.test_add(5); },
    test_sub_three => { s.test_sub(3); },
    test_mul_by_four => { s.test_mul(4); },
    test_div_by_two => { s.test_div(2); },
    test_mod_three => { s.test_mod(3); },
    test_negate => { s.test_negate(); },
    test_square => { s.test_square(); },
    test_cube => { s.test_cube(); },
    test_increment_by_ten => { s.test_increment_by_ten(); },
);

fn quickstart_body() {
    let mut suite = Counter::default();
    run(&mut suite, MY_CASES, RunConfig::all(), &MY_HOOKS);
}

fn main() {
    quickstart_body();
}
