//! Ensures [`test_suite!`] shares one suite across two generated `#[test]`s: the first case seeds
//! state, the second reads and updates it on the **same** `Mutex` value.

#![allow(unused_imports)]

use crate::{cases, test_suite, Case, HookFns, RunConfig, run};

#[derive(Default, Debug)]
struct SharedState {
    phase: &'static str,
    counter: i32,
}

static SHARED_CASES: &[Case<SharedState>] = cases![SharedState, s =>
    init_state => {
        assert_eq!(s.phase, "", "fresh suite");
        s.phase = "initialized";
        s.counter = 42;
    },
    mutate_after_init => {
        assert_eq!(s.phase, "initialized", "same suite as init_state");
        assert_eq!(s.counter, 42);
        s.counter += 1;
        s.phase = "touched_twice";
    },
];

static SHARED_HOOKS: HookFns<SharedState> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

test_suite!(
    SharedState,
    SHARED_STATE_TEST_SUITE,
    SharedState::default(),
    SHARED_CASES,
    SHARED_HOOKS,
    [init_state, mutate_after_init]
);

/// Control: one `run` with [`RunConfig::all`] applies both cases in order on a single suite (no `Mutex` split).
#[test]
fn sequential_run_sets_then_mutates_without_test_suite_split() {
    let mut suite = SharedState::default();
    run(
        &mut suite,
        SHARED_CASES,
        RunConfig::all(),
        &SHARED_HOOKS,
    );
    assert_eq!(suite.phase, "touched_twice");
    assert_eq!(suite.counter, 43);
}
