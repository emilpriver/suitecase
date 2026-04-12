#[track_caller]
pub fn true_(value: bool) {
    true_with_msg(value, "");
}

#[track_caller]
pub fn true_with_msg(value: bool, msg: &str) {
    if !value {
        if msg.is_empty() {
            panic!("assertion failed: `true_`");
        }
        panic!("assertion failed: `true_`: {msg}");
    }
}

#[track_caller]
pub fn false_(value: bool) {
    false_with_msg(value, "");
}

#[track_caller]
pub fn false_with_msg(value: bool, msg: &str) {
    if value {
        if msg.is_empty() {
            panic!("assertion failed: `false_`");
        }
        panic!("assertion failed: `false_`: {msg}");
    }
}
