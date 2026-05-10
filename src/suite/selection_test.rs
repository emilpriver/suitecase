use crate::{Case, HookFns, RunConfig, run, fail, fail_now};

#[derive(Default)]
struct SelRecorder {
    log: Vec<&'static str>,
}

impl SelRecorder {
    fn push(&mut self, e: &'static str) {
        self.log.push(e);
    }
}

fn sr_setup(s: &mut SelRecorder) {
    s.push("setup_suite");
}
fn sr_teardown(s: &mut SelRecorder) {
    s.push("teardown_suite");
}
fn sr_before(s: &mut SelRecorder) {
    s.push("before_each");
}
fn sr_after(s: &mut SelRecorder) {
    s.push("after_each");
}

static SEL_HOOKS: HookFns<SelRecorder> = HookFns {
    setup_suite: Some(sr_setup),
    teardown_suite: Some(sr_teardown),
    before_each: Some(sr_before),
    after_each: Some(sr_after),
};

#[test]
fn single_filter_runs_only_that_case() {
    let mut suite = SelRecorder::default();
    let cases: &[Case<SelRecorder>] = cases![SelRecorder, s =>
        first => { s.push("first"); },
        second => { s.push("second"); },
        third => { s.push("third"); },
    ];
    run(&mut suite, cases, RunConfig::filter("second"), &SEL_HOOKS);
    assert_eq!(
        suite.log,
        vec![
            "setup_suite",
            "before_each",
            "second",
            "after_each",
            "teardown_suite",
        ]
    );
}

#[test]
fn multiple_filters_run_matching_cases() {
    let mut suite = SelRecorder::default();
    let cases: &[Case<SelRecorder>] = cases![SelRecorder, s =>
        alpha => { s.push("alpha"); },
        beta => { s.push("beta"); },
        gamma => { s.push("gamma"); },
    ];
    run(
        &mut suite,
        cases,
        RunConfig::filters(["alpha".to_string(), "gamma".to_string()]),
        &SEL_HOOKS,
    );
    assert_eq!(
        suite.log,
        vec![
            "setup_suite",
            "before_each",
            "alpha",
            "after_each",
            "before_each",
            "gamma",
            "after_each",
            "teardown_suite",
        ]
    );
}

#[test]
fn glob_pattern_matches_cases() {
    let mut suite = SelRecorder::default();
    let cases: &[Case<SelRecorder>] = cases![SelRecorder, s =>
        test_a => { s.push("test_a"); },
        test_b => { s.push("test_b"); },
        other => { s.push("other"); },
    ];
    run(
        &mut suite,
        cases,
        RunConfig::pattern("test_*"),
        &SEL_HOOKS,
    );
    assert_eq!(
        suite.log,
        vec![
            "setup_suite",
            "before_each",
            "test_a",
            "after_each",
            "before_each",
            "test_b",
            "after_each",
            "teardown_suite",
        ]
    );
}

#[test]
fn glob_pattern_prefix_match() {
    let mut suite = SelRecorder::default();
    let cases: &[Case<SelRecorder>] = cases![SelRecorder, s =>
        setup_db => { s.push("setup_db"); },
        setup_cache => { s.push("setup_cache"); },
        teardown => { s.push("teardown"); },
    ];
    run(
        &mut suite,
        cases,
        RunConfig::pattern("setup_*"),
        &SEL_HOOKS,
    );
    assert_eq!(
        suite.log,
        vec![
            "setup_suite",
            "before_each",
            "setup_db",
            "after_each",
            "before_each",
            "setup_cache",
            "after_each",
            "teardown_suite",
        ]
    );
}

#[test]
fn depends_on_runs_deps_before_target() {
    let mut suite = SelRecorder::default();
    let cases: &[Case<SelRecorder>] = cases![SelRecorder, s =>
        setup => { s.push("setup"); },
        action(depends_on = [setup]) => { s.push("action"); },
        verify(depends_on = [action]) => { s.push("verify"); },
    ];
    run(
        &mut suite,
        cases,
        RunConfig::filter("verify"),
        &SEL_HOOKS,
    );
    assert_eq!(
        suite.log,
        vec![
            "setup_suite",
            "before_each",
            "setup",
            "after_each",
            "before_each",
            "action",
            "after_each",
            "before_each",
            "verify",
            "after_each",
            "teardown_suite",
        ]
    );
}

#[test]
fn depends_on_multiple_deps() {
    let mut suite = SelRecorder::default();
    let cases: &[Case<SelRecorder>] = cases![SelRecorder, s =>
        a => { s.push("a"); },
        b => { s.push("b"); },
        c(depends_on = [a, b]) => { s.push("c"); },
    ];
    run(
        &mut suite,
        cases,
        RunConfig::filter("c"),
        &SEL_HOOKS,
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
            "before_each",
            "c",
            "after_each",
            "teardown_suite",
        ]
    );
}

#[test]
#[should_panic(expected = "dependency")]
fn missing_dep_panics() {
    let mut suite = SelRecorder::default();
    let cases: &[Case<SelRecorder>] = cases![SelRecorder, s =>
        only(depends_on = [nonexistent]) => { s.push("only"); },
    ];
    run(
        &mut suite,
        cases,
        RunConfig::filter("only"),
        &SEL_HOOKS,
    );
}

#[test]
#[should_panic(expected = "circular dependency")]
fn circular_dep_panics() {
    let mut suite = SelRecorder::default();
    let a = Case::<SelRecorder> { name: "a", run: |s| s.push("a"), dependencies: &["b"] };
    let b = Case::<SelRecorder> { name: "b", run: |s| s.push("b"), dependencies: &["a"] };
    let cases: &[Case<SelRecorder>] = &[a, b];
    run(
        &mut suite,
        cases,
        RunConfig::filter("a"),
        &SEL_HOOKS,
    );
}

#[test]
fn fail_in_dep_skips_dependents() {
    let mut suite = SelRecorder::default();
    let cases: &[Case<SelRecorder>] = cases![SelRecorder, s =>
        setup => { fail("setup failed"); },
        action(depends_on = [setup]) => { s.push("action"); },
        verify(depends_on = [action]) => { s.push("verify"); },
    ];
    run(&mut suite, cases, RunConfig::all(), &SEL_HOOKS);
    assert_eq!(
        suite.log,
        vec![
            "setup_suite",
            "before_each",
            "after_each",
            "teardown_suite",
        ]
    );
}

#[test]
fn fail_now_in_dep_aborts_and_skips_dependents() {
    let mut suite = SelRecorder::default();
    let cases: &[Case<SelRecorder>] = cases![SelRecorder, s =>
        setup => { fail_now("setup abort"); },
        action(depends_on = [setup]) => { s.push("action"); },
    ];
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run(&mut suite, cases, RunConfig::all(), &SEL_HOOKS);
    }));
    assert!(err.is_err());
    assert_eq!(
        suite.log,
        vec![
            "setup_suite",
            "before_each",
            "after_each",
            "teardown_suite",
        ]
    );
}

#[test]
fn depends_on_preserves_topological_order() {
    let mut suite = SelRecorder::default();
    let cases: &[Case<SelRecorder>] = cases![SelRecorder, s =>
        z => { s.push("z"); },
        a => { s.push("a"); },
        b(depends_on = [a]) => { s.push("b"); },
    ];
    run(
        &mut suite,
        cases,
        RunConfig::filters(["z".to_string(), "b".to_string()]),
        &SEL_HOOKS,
    );
    assert_eq!(
        suite.log,
        vec![
            "setup_suite",
            "before_each",
            "z",
            "after_each",
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
