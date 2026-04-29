use suitecase::{test_suite, HookFns, Case, cases, run, RunConfig};

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
    test_failing,
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
