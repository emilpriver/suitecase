//! Boolean assertions for test code.

/// Asserts that `value` is `true`.
///
/// # Panics
///
/// Panics if `value` is `false`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::true_;
///
/// true_(1 < 2);
/// ```
#[track_caller]
pub fn true_(value: bool) {
    true_with_msg(value, "");
}

/// Like [`true_`], but includes `msg` in the panic message when the assertion fails.
///
/// # Panics
///
/// Panics if `value` is `false`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::true_with_msg;
///
/// true_with_msg(2 + 2 == 4, "arithmetic");
/// ```
#[track_caller]
pub fn true_with_msg(value: bool, msg: &str) {
    if !value {
        if msg.is_empty() {
            panic!("assertion failed: `true_`");
        }
        panic!("assertion failed: `true_`: {msg}");
    }
}

/// Asserts that `value` is `false`.
///
/// # Panics
///
/// Panics if `value` is `true`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::false_;
///
/// false_(1 > 2);
/// ```
#[track_caller]
pub fn false_(value: bool) {
    false_with_msg(value, "");
}

/// Like [`false_`], but includes `msg` in the panic message when the assertion fails.
///
/// # Panics
///
/// Panics if `value` is `true`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::false_with_msg;
///
/// false_with_msg(2 + 2 == 5, "still math");
/// ```
#[track_caller]
pub fn false_with_msg(value: bool, msg: &str) {
    if value {
        if msg.is_empty() {
            panic!("assertion failed: `false_`");
        }
        panic!("assertion failed: `false_`: {msg}");
    }
}
