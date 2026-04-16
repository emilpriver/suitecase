use core::panic;

use crate::{Case, HookFns, RunConfig, run};

#[derive(Default)]
struct Recorder {
    log: Vec<&'static str>,
}

impl Recorder {
    fn push(&mut self, e: &'static str) {
        self.log.push(e);
    }
}

impl Recorder {
    fn test_a(&mut self) {
        self.push("a");
    }

    fn test_b(&mut self) {
        self.push("b");
    }

    fn test_only(&mut self) {
        self.push("only");
    }

    fn test_bad(&mut self) {
        let _ = self;
        panic!("boom");
    }
}

fn rec_setup(s: &mut Recorder) {
    s.push("setup_suite");
}
fn rec_teardown(s: &mut Recorder) {
    s.push("teardown_suite");
}
fn rec_before(s: &mut Recorder) {
    s.push("before_each");
}
fn rec_after(s: &mut Recorder) {
    s.push("after_each");
}

static RECORDER_HOOKS: HookFns<Recorder> = HookFns {
    setup_suite: Some(rec_setup),
    teardown_suite: Some(rec_teardown),
    before_each: Some(rec_before),
    after_each: Some(rec_after),
};

static RECORDER_CASES: &[Case<Recorder>] = cases![Recorder, s =>
    test_a => { s.test_a(); },
    test_b => { s.test_b(); },
];

#[derive(Default)]
struct DefaultsOnly {
    n: i32,
}

impl DefaultsOnly {
    fn test_one(&mut self) {
        self.n = 7;
    }
}

static DEFAULTS_HOOKS: HookFns<DefaultsOnly> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

test_suite!(
    DefaultsOnly,
    DEFAULTS_SUITE,
    DEFAULTS_CURSOR,
    DEFAULTS_CASES,
    DefaultsOnly::default(),
    DEFAULTS_HOOKS,
    s =>
    test_one => { s.test_one(); },
);

test_suite!(
    Recorder,
    REC_SUITE,
    REC_CURSOR,
    Recorder::default(),
    RECORDER_CASES,
    RECORDER_HOOKS,
    [test_a, test_b]
);

/// One generated test; exercises `test_suite!` (shared `Mutex` suite path).
#[derive(Default)]
struct SharedSmoke(u8);

impl SharedSmoke {
    fn test_shared_smoke(&mut self) {
        self.0 = 1;
    }
}

static SHARED_SMOKE_CASES: &[Case<SharedSmoke>] = cases![SharedSmoke, s =>
    test_shared_smoke => { s.test_shared_smoke(); },
];

static SHARED_SMOKE_HOOKS: HookFns<SharedSmoke> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

test_suite!(
    SharedSmoke,
    SHARED_SMOKE_SUITE,
    SHARED_SMOKE_CURSOR,
    SharedSmoke::default(),
    SHARED_SMOKE_CASES,
    SHARED_SMOKE_HOOKS,
    [test_shared_smoke]
);

fn case_fn_a(s: &mut Recorder) {
    s.push("a");
}
fn case_fn_b(s: &mut Recorder) {
    s.push("b");
}

static RECORDER_FN_CASES: &[Case<Recorder>] = cases![Recorder, s =>
    suite_cf_a => { case_fn_a(s); },
    suite_cf_b => { case_fn_b(s); },
];

test_suite!(
    Recorder,
    REC_FN_SUITE,
    REC_FN_CURSOR,
    Recorder::default(),
    RECORDER_FN_CASES,
    RECORDER_HOOKS,
    [suite_cf_a, suite_cf_b]
);

#[test]
fn default_hooks_run_cases_only() {
    let mut suite = DefaultsOnly::default();
    run(
        &mut suite,
        DEFAULTS_CASES,
        RunConfig::all(),
        &HookFns::default(),
    );
    assert_eq!(suite.n, 7);
}

#[test]
fn hook_order_all_cases() {
    let mut suite = Recorder::default();
    run(
        &mut suite,
        RECORDER_CASES,
        RunConfig::all(),
        &RECORDER_HOOKS,
    );

    assert_eq!(
        suite.log,
        vec![
            "setup_suite",
            "before_each",
            "a",
            "after_each",
            "before_each",
            "b",
            "after_each",
            "teardown_suite",
        ]
    );
}

#[test]
fn singular_filtered_case_runs_setup_and_teardown() {
    let mut suite = Recorder::default();
    run(
        &mut suite,
        RECORDER_CASES,
        RunConfig::filter("test_b"),
        &RECORDER_HOOKS,
    );

    assert_eq!(
        suite.log,
        vec![
            "setup_suite",
            "before_each",
            "b",
            "after_each",
            "teardown_suite",
        ]
    );
}

#[test]
fn cases_inline_free_functions() {
    let mut suite = Recorder::default();
    run(
        &mut suite,
        RECORDER_FN_CASES,
        RunConfig::all(),
        &RECORDER_HOOKS,
    );
    assert_eq!(
        suite.log,
        vec![
            "setup_suite",
            "before_each",
            "a",
            "after_each",
            "before_each",
            "b",
            "after_each",
            "teardown_suite",
        ]
    );
}

#[test]
fn no_cases_no_setup() {
    let mut suite = Recorder::default();
    run(&mut suite, &[], RunConfig::all(), &RECORDER_HOOKS);
    assert!(suite.log.is_empty());
}

#[test]
#[should_panic(expected = "matched no cases")]
fn invalid_filter_panics() {
    let mut suite = Recorder::default();
    static ONLY_CASE: &[Case<Recorder>] = cases![Recorder, s =>
        test_only => { s.test_only(); },
    ];
    run(
        &mut suite,
        ONLY_CASE,
        RunConfig::filter("nope"),
        &RECORDER_HOOKS,
    );
}

#[test]
fn after_each_runs_when_case_panics() {
    let mut suite = Recorder::default();
    static BAD_CASE: &[Case<Recorder>] = cases![Recorder, s =>
        test_bad => { s.test_bad(); },
    ];
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run(&mut suite, BAD_CASE, RunConfig::all(), &RECORDER_HOOKS);
    }));
    assert!(err.is_err());
    assert_eq!(
        suite.log,
        vec!["setup_suite", "before_each", "after_each", "teardown_suite",]
    );
}

#[test]
fn partial_hooks_only_run_some() {
    let mut suite = Recorder::default();
    static ONE_CASE: &[Case<Recorder>] = cases![Recorder, s =>
        test_a => { s.test_a(); },
    ];
    run(
        &mut suite,
        ONE_CASE,
        RunConfig::all(),
        &HookFns {
            setup_suite: Some(rec_setup),
            teardown_suite: None,
            before_each: Some(rec_before),
            after_each: None,
        },
    );
    assert_eq!(suite.log, vec!["setup_suite", "before_each", "a"]);
}
