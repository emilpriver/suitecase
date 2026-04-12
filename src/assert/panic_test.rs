use super::panic::*;

#[test]
fn panics_ok() {
    panics(|| panic!("boom"));
}

#[test]
#[should_panic(expected = "`panics`")]
fn panics_fails() {
    panics(|| {});
}

#[test]
fn not_panics_ok() {
    not_panics(|| {
        let _ = 1 + 1;
    });
}

#[test]
#[should_panic(expected = "`not_panics`")]
fn not_panics_fails() {
    not_panics(|| panic!("x"));
}

#[test]
fn panics_with_substring_ok() {
    panics_with_substring(|| panic!("expected token"), "token");
}
