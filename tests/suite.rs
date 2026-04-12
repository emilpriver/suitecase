use suitecase::{
    Case, HookFns, RunConfig, cargo_case_tests, cargo_case_tests_with_hooks, cases_fn, run,
    suite_methods,
};

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

static RECORDER_CASES: &[Case<Recorder>] = suite_methods![Recorder, s => test_a, test_b];

#[derive(Default)]
struct DefaultsOnly {
    n: i32,
}

impl DefaultsOnly {
    fn test_one(&mut self) {
        self.n = 7;
    }
}

static DEFAULTS_CASES: &[Case<DefaultsOnly>] = suite_methods![DefaultsOnly, s => test_one];

cargo_case_tests!(DefaultsOnly::default(), DEFAULTS_CASES, [test_one]);

cargo_case_tests_with_hooks!(
    Recorder::default(),
    RECORDER_CASES,
    RECORDER_HOOKS,
    [test_a, test_b]
);

fn case_fn_a(s: &mut Recorder) {
    s.push("a");
}
fn case_fn_b(s: &mut Recorder) {
    s.push("b");
}

static RECORDER_FN_CASES: &[Case<Recorder>] = cases_fn![
    Recorder =>
    suite_cf_a => case_fn_a,
    suite_cf_b => case_fn_b
];

cargo_case_tests_with_hooks!(
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
        suite_methods![Recorder, s => test_a, test_b],
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
        suite_methods![Recorder, s => test_a, test_b],
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
fn cases_fn_macro_works() {
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
    run(
        &mut suite,
        suite_methods![Recorder, s => test_only],
        RunConfig::filter("nope"),
        &RECORDER_HOOKS,
    );
}

#[test]
fn after_each_runs_when_case_panics() {
    let mut suite = Recorder::default();
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run(
            &mut suite,
            suite_methods![Recorder, s => test_bad],
            RunConfig::all(),
            &RECORDER_HOOKS,
        );
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
    run(
        &mut suite,
        suite_methods![Recorder, s => test_a],
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
