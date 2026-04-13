use super::inject_message_format_before_libtest_split;
use std::ffi::OsString;

#[test]
fn inject_inserts_before_double_dash() {
    let mut args = vec![
        OsString::from("-p"),
        OsString::from("suitecase"),
        OsString::from("--"),
        OsString::from("--nocapture"),
    ];
    inject_message_format_before_libtest_split(&mut args);
    assert_eq!(
        args,
        vec![
            OsString::from("-p"),
            OsString::from("suitecase"),
            OsString::from("--message-format"),
            OsString::from("json-diagnostic-rendered-ansi"),
            OsString::from("--"),
            OsString::from("--nocapture"),
        ]
    );
}

#[test]
fn inject_skips_when_message_format_present() {
    let mut args = vec![OsString::from("--message-format"), OsString::from("short")];
    inject_message_format_before_libtest_split(&mut args);
    assert_eq!(
        args,
        vec![OsString::from("--message-format"), OsString::from("short"),]
    );
}
