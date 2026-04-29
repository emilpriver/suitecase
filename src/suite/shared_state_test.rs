//! Two cases share one [`SharedState`] behind a single `#[test]` via [`test_suite!`].
//! Cases run sequentially in slice order — no polling, no serial_test, no ordering ambiguity.
use crate::{Case, HookFns, RunConfig, run};

#[derive(Default, Debug)]
struct SharedState {
    counter: i32,
}

static SHARED_CASES: &[Case<SharedState>] = cases![SharedState, s =>
    init_state => {
        assert_eq!(s.counter, 0, "fresh shared suite");
        s.counter = 42;
    },
    mutate_after_init => {
        assert_eq!(s.counter, 42, "same suite as init_state");
        s.counter += 1;
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
    SHARED_SUITE,
    shared_state_test_suite_single_run,
    SharedState::default(),
    SHARED_HOOKS,
    s =>
    init_state => {
        assert_eq!(s.counter, 0, "fresh shared suite");
        s.counter = 42;
    },
    mutate_after_init => {
        assert_eq!(s.counter, 42, "same suite as init_state");
        s.counter += 1;
    },
);

/// Same cases in one `run` with [`RunConfig::all`] (single suite, ordered cases).
#[test]
fn sequential_run_matches_shared_increment() {
    let mut suite = SharedState::default();
    run(&mut suite, SHARED_CASES, RunConfig::all(), &SHARED_HOOKS);
    assert_eq!(suite.counter, 43);
}
