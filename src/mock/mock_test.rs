use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::{
    Mock, TestingT, anything, anything_of_type, assert_expectations_for_objects, eq, matched_by,
};

struct NoopT;
impl TestingT for NoopT {
    fn errorf(&self, _msg: &str) {}
    fn fail_now(&self) {}
}

struct RecordingT {
    messages: Mutex<Vec<String>>,
    fail_now_hits: AtomicUsize,
}

impl RecordingT {
    fn new() -> Self {
        Self {
            messages: Mutex::new(Vec::new()),
            fail_now_hits: AtomicUsize::new(0),
        }
    }
}

impl TestingT for RecordingT {
    fn errorf(&self, msg: &str) {
        self.messages.lock().unwrap().push(msg.to_string());
    }

    fn fail_now(&self) {
        self.fail_now_hits.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn expectation_met_finish() {
    let m = Mock::new();
    m.on("do_it", vec![eq(1i32)])
        .returning(|| vec![Box::new(true), Box::new(false)])
        .finish();
    let out = m.method_called("do_it", crate::mock_args!(1i32));
    assert!(out.bool(0));
    assert!(!out.bool(1));
    assert!(m.assert_expectations(&NoopT));
}

#[test]
fn assert_expectations_fails_when_required_never_invoked() {
    let m = Mock::new();
    m.on("idle", vec![])
        .returning(|| vec![Box::new(0u8)])
        .finish();
    let t = RecordingT::new();
    assert!(!m.assert_expectations(&t));
    assert_eq!(t.fail_now_hits.load(Ordering::SeqCst), 1);
    let msgs = t.messages.lock().unwrap();
    assert_eq!(msgs.len(), 1);
    assert!(
        msgs[0].contains("idle"),
        "msg should mention method: {:?}",
        msgs[0]
    );
    assert!(msgs[0].contains("0 invocations"), "{:?}", msgs[0]);
}

#[test]
fn assert_expectations_passes_for_maybe_without_call() {
    let m = Mock::new();
    m.on("opt", vec![]).returning(|| vec![Box::new(())]).maybe();
    assert!(m.assert_expectations(&NoopT));
}

#[test]
fn assert_expectations_ignores_removed_expectation() {
    let m = Mock::new();
    let h = m
        .on("gone", vec![])
        .returning(|| vec![Box::new(0u8)])
        .finish();
    h.unset();
    assert!(m.assert_expectations(&NoopT));
}

#[test]
fn anything_matcher() {
    let m = Mock::new();
    m.on("x", vec![anything()])
        .returning(|| vec![Box::new(())])
        .once();
    m.method_called("x", crate::mock_args!("hello"));
    assert!(m.assert_expectations(&NoopT));
}

#[test]
fn assert_expectations_for_objects_ok() {
    let a = Mock::new();
    let b = Mock::new();
    a.on("a", vec![])
        .returning(|| vec![Box::new(0i32)])
        .finish();
    b.on("b", vec![])
        .returning(|| vec![Box::new(0i32)])
        .finish();
    a.method_called("a", crate::mock_args!());
    b.method_called("b", crate::mock_args!());
    assert!(assert_expectations_for_objects(&NoopT, &[&a, &b]));
}

#[test]
fn assert_expectations_for_objects_fails_on_first_mock() {
    let a = Mock::new();
    let b = Mock::new();
    a.on("a", vec![])
        .returning(|| vec![Box::new(0i32)])
        .finish();
    b.on("b", vec![])
        .returning(|| vec![Box::new(0i32)])
        .finish();
    a.method_called("a", crate::mock_args!());
    let t = RecordingT::new();
    assert!(!assert_expectations_for_objects(&t, &[&a, &b]));
    assert_eq!(t.fail_now_hits.load(Ordering::SeqCst), 1);
}

#[test]
#[should_panic(expected = "unexpected call")]
fn unexpected_call_panics() {
    let m = Mock::new();
    m.method_called("nope", crate::mock_args!());
}

#[test]
#[should_panic(expected = "unexpected call")]
fn once_exhausted_then_second_call_panics() {
    let m = Mock::new();
    m.on("one", vec![eq(0i32)])
        .returning(|| vec![Box::new(())])
        .once();
    m.method_called("one", crate::mock_args!(0i32));
    m.method_called("one", crate::mock_args!(0i32));
}

#[test]
fn times_requires_exact_invocation_count() {
    let m = Mock::new();
    m.on("n", vec![]).returning(|| vec![Box::new(0u8)]).times(2);
    m.method_called("n", crate::mock_args!());
    assert!(!m.assert_expectations(&RecordingT::new()));
    m.method_called("n", crate::mock_args!());
    assert!(m.assert_expectations(&NoopT));
}

#[test]
fn twice_matches_two_calls() {
    let m = Mock::new();
    m.on("t", vec![]).returning(|| vec![Box::new(())]).twice();
    m.method_called("t", crate::mock_args!());
    assert!(!m.assert_expectations(&RecordingT::new()));
    m.method_called("t", crate::mock_args!());
    assert!(m.assert_expectations(&NoopT));
}

#[test]
fn assert_called_finds_matching_record() {
    let m = Mock::new();
    m.on("f", vec![eq(7i32)])
        .returning(|| vec![Box::new(())])
        .finish();
    m.method_called("f", crate::mock_args!(7i32));
    assert!(m.assert_called(&NoopT, "f", &[eq(7i32)]));
}

#[test]
fn assert_called_fails_without_match() {
    let m = Mock::new();
    m.on("f", vec![eq(1i32)])
        .returning(|| vec![Box::new(())])
        .finish();
    m.method_called("f", crate::mock_args!(1i32));
    let t = RecordingT::new();
    assert!(!m.assert_called(&t, "f", &[eq(2i32)]));
    assert_eq!(t.fail_now_hits.load(Ordering::SeqCst), 1);
}

#[test]
fn assert_not_called_passes_when_no_match() {
    let m = Mock::new();
    m.on("f", vec![anything()])
        .returning(|| vec![Box::new(())])
        .finish();
    m.method_called("f", crate::mock_args!(2i32));
    assert!(m.assert_not_called(&NoopT, "f", &[eq(1i32)]));
}

#[test]
fn assert_not_called_fails_when_match_exists() {
    let m = Mock::new();
    m.on("f", vec![eq(1i32)])
        .returning(|| vec![Box::new(())])
        .finish();
    m.method_called("f", crate::mock_args!(1i32));
    let t = RecordingT::new();
    assert!(!m.assert_not_called(&t, "f", &[eq(1i32)]));
    assert_eq!(t.fail_now_hits.load(Ordering::SeqCst), 1);
}

#[test]
fn assert_number_of_calls_counts_by_method_name() {
    let m = Mock::new();
    m.on("g", vec![]).returning(|| vec![Box::new(())]).finish();
    m.on("g", vec![]).returning(|| vec![Box::new(())]).finish();
    m.method_called("g", crate::mock_args!());
    m.method_called("g", crate::mock_args!());
    assert!(m.assert_number_of_calls(&NoopT, "g", 2));
}

#[test]
fn assert_number_of_calls_fails_on_wrong_count() {
    let m = Mock::new();
    m.on("g", vec![]).returning(|| vec![Box::new(())]).finish();
    m.method_called("g", crate::mock_args!());
    let t = RecordingT::new();
    assert!(!m.assert_number_of_calls(&t, "g", 2));
    assert_eq!(t.fail_now_hits.load(Ordering::SeqCst), 1);
}

#[test]
fn run_hook_runs_before_return() {
    let m = Mock::new();
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_run = hits.clone();
    m.on("r", vec![eq(5i32)])
        .returning(|| vec![Box::new(99i32)])
        .run(move |args| {
            assert_eq!(args.int(0), 5);
            hits_run.fetch_add(1, Ordering::SeqCst);
        })
        .once();
    let out = m.method_called("r", crate::mock_args!(5i32));
    assert_eq!(out.int(0), 99);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[test]
fn anything_of_type_matches_concrete_type() {
    let m = Mock::new();
    m.on("t", vec![anything_of_type::<i32>()])
        .returning(|| vec![Box::new(())])
        .once();
    m.method_called("t", crate::mock_args!(42i32));
    assert!(m.assert_expectations(&NoopT));
}

#[test]
fn matched_by_custom_predicate() {
    let m = Mock::new();
    m.on(
        "t",
        vec![matched_by(|a| a.downcast_ref::<i32>() == Some(&3))],
    )
    .returning(|| vec![Box::new(())])
    .once();
    m.method_called("t", crate::mock_args!(3i32));
    assert!(m.assert_expectations(&NoopT));
}

#[test]
#[should_panic(expected = "boom")]
fn panic_expectation_panics_in_method_called() {
    let m = Mock::new();
    m.on("p", vec![])
        .returning(|| vec![Box::new(())])
        .panic("boom");
    m.method_called("p", crate::mock_args!());
}

#[test]
fn unset_allows_re_registering_same_method() {
    let m = Mock::new();
    let h = m
        .on("m", vec![eq(1i32)])
        .returning(|| vec![Box::new(10i32)])
        .finish();
    h.unset();
    m.on("m", vec![eq(2i32)])
        .returning(|| vec![Box::new(20i32)])
        .finish();
    let out = m.method_called("m", crate::mock_args!(2i32));
    assert_eq!(out.int(0), 20);
    assert!(m.assert_expectations(&NoopT));
}
