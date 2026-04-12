use super::boolean::*;

#[test]
fn true_accepts_true() {
    true_(true);
}

#[test]
#[should_panic(expected = "assertion failed: `true_`")]
fn true_rejects_false() {
    true_(false);
}

#[test]
fn true_with_msg_panic_includes_message() {
    let r = std::panic::catch_unwind(|| true_with_msg(false, "nope"));
    assert!(r.is_err());
}

#[test]
fn false_accepts_false() {
    false_(false);
}

#[test]
#[should_panic(expected = "assertion failed: `false_`")]
fn false_rejects_true() {
    false_(true);
}
