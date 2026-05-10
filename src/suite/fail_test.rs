use crate::{Case, HookFns, RunConfig, run, fail, fail_now};

#[derive(Default)]
struct FailRecorder {
    log: Vec<&'static str>,
}

impl FailRecorder {
    fn push(&mut self, e: &'static str) {
        self.log.push(e);
    }
}

fn fr_setup(s: &mut FailRecorder) {
    s.push("setup_suite");
}
fn fr_teardown(s: &mut FailRecorder) {
    s.push("teardown_suite");
}

static FAIL_HOOKS: HookFns<FailRecorder> = HookFns {
    setup_suite: Some(fr_setup),
    teardown_suite: Some(fr_teardown),
    before_each: None,
    after_each: None,
};

#[test]
fn fail_marks_case_failed_and_continues() {
    let mut suite = FailRecorder::default();
    static CASES: &[Case<FailRecorder>] = cases![FailRecorder, s =>
        first => { s.push("first"); },
        second => { fail("expected failure"); },
        third => { s.push("third"); },
    ];
    run(&mut suite, CASES, RunConfig::all(), &FAIL_HOOKS);
    assert_eq!(
        suite.log,
        vec!["setup_suite", "first", "third", "teardown_suite"]
    );
}

#[test]
fn fail_now_aborts_remaining_cases() {
    let mut suite = FailRecorder::default();
    static CASES: &[Case<FailRecorder>] = cases![FailRecorder, s =>
        first => { s.push("first"); },
        second => { fail_now("abort here"); },
        third => { s.push("third"); },
    ];
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run(&mut suite, CASES, RunConfig::all(), &FAIL_HOOKS);
    }));
    assert!(err.is_err());
    assert_eq!(
        suite.log,
        vec!["setup_suite", "first", "teardown_suite"]
    );
}

#[test]
fn fail_now_runs_teardown() {
    let mut suite = FailRecorder::default();
    static CASES: &[Case<FailRecorder>] = cases![FailRecorder, s =>
        only => { fail_now("abort"); },
    ];
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run(&mut suite, CASES, RunConfig::all(), &FAIL_HOOKS);
    }));
    assert!(err.is_err());
    assert_eq!(suite.log, vec!["setup_suite", "teardown_suite"]);
}

#[test]
fn multiple_fail_only_first_records() {
    let mut suite = FailRecorder::default();
    static CASES: &[Case<FailRecorder>] = cases![FailRecorder, s =>
        a => { fail("first fail"); },
        b => { fail("second fail"); },
        c => { s.push("c"); },
    ];
    run(&mut suite, CASES, RunConfig::all(), &FAIL_HOOKS);
    assert_eq!(
        suite.log,
        vec!["setup_suite", "c", "teardown_suite"]
    );
}

#[test]
fn regular_panic_still_re_raises() {
    let mut suite = FailRecorder::default();
    static CASES: &[Case<FailRecorder>] = cases![FailRecorder, s =>
        first => { s.push("first"); },
        second => { panic!("regular panic"); },
        third => { s.push("third"); },
    ];
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run(&mut suite, CASES, RunConfig::all(), &FAIL_HOOKS);
    }));
    assert!(err.is_err());
    assert_eq!(
        suite.log,
        vec!["setup_suite", "first", "third", "teardown_suite"]
    );
}
