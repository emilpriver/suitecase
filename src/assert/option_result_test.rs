use super::option_result::*;

#[test]
fn is_none_ok() {
    is_none(&None::<i32>);
}

#[test]
#[should_panic(expected = "is_none")]
fn is_none_fails() {
    is_none(&Some(1));
}

#[test]
fn is_some_ok() {
    is_some(&Some(1));
}

#[test]
#[should_panic(expected = "is_some")]
fn is_some_fails() {
    is_some(&None::<i32>);
}

#[test]
fn unwrap_some_ok() {
    assert_eq!(unwrap_some(Some(7)), 7);
}

#[test]
fn zero_ok() {
    zero(&0_i32);
}

#[test]
#[should_panic(expected = "`zero`")]
fn zero_fails() {
    zero(&1_i32);
}

#[test]
fn not_zero_ok() {
    not_zero(&1_i32);
}

#[test]
#[should_panic(expected = "`not_zero`")]
fn not_zero_fails() {
    not_zero(&0_i32);
}

#[test]
fn assert_ok_ok() {
    assert_eq!(assert_ok(Ok::<_, ()>(9)), 9);
}

#[test]
#[should_panic(expected = "`assert_ok`")]
fn assert_ok_fails() {
    assert_ok(Err::<i32, _>("e"));
}

#[test]
fn no_error_ok() {
    assert_eq!(no_error(Ok::<i32, std::convert::Infallible>(3)), 3);
}

#[test]
fn assert_err_ok() {
    let e = assert_err::<i32, &str>(Err("bad"));
    assert_eq!(e, "bad");
}

#[test]
#[should_panic(expected = "`assert_err`")]
fn assert_err_fails() {
    assert_err(Ok::<i32, &str>(1));
}
