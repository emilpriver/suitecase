//! Two `#[test]` functions share one [`SharedState`] behind [`std::sync::Mutex`] in a
//! [`std::sync::OnceLock`]: `shared_init_state` runs only the `init_state` case, then
//! `shared_mutate_after_init` waits (without holding the mutex) until that finishes, then runs
//! `mutate_after_init`. This stays reliable under the default **parallel** test harness.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

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

static SHARED_SUITE: OnceLock<Mutex<SharedState>> = OnceLock::new();
static INIT_CASE_FINISHED: AtomicBool = AtomicBool::new(false);

#[test]
fn shared_init_state() {
    INIT_CASE_FINISHED.store(false, Ordering::SeqCst);
    let mut suite = SHARED_SUITE
        .get_or_init(|| Mutex::new(SharedState::default()))
        .lock()
        .expect("suite mutex poisoned");
    run(
        &mut *suite,
        SHARED_CASES,
        RunConfig::filter("init_state"),
        &SHARED_HOOKS,
    );
    INIT_CASE_FINISHED.store(true, Ordering::SeqCst);
}

#[test]
fn shared_mutate_after_init() {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !INIT_CASE_FINISHED.load(Ordering::SeqCst) {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for shared_init_state — run both tests in one cargo test invocation"
        );
        thread::yield_now();
    }
    let mut suite = SHARED_SUITE
        .get_or_init(|| Mutex::new(SharedState::default()))
        .lock()
        .expect("suite mutex poisoned");
    run(
        &mut *suite,
        SHARED_CASES,
        RunConfig::filter("mutate_after_init"),
        &SHARED_HOOKS,
    );
}

/// Same cases in one `run` with [`RunConfig::all`] (single suite, ordered cases).
#[test]
fn sequential_run_matches_shared_increment() {
    let mut suite = SharedState::default();
    run(&mut suite, SHARED_CASES, RunConfig::all(), &SHARED_HOOKS);
    assert_eq!(suite.counter, 43);
}
