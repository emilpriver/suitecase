use std::error::Error;
use std::fmt::Debug;

#[track_caller]
pub fn is_error<T: Debug, E>(result: &Result<T, E>) {
    if result.is_ok() {
        panic!(
            "assertion failed: `is_error`: expected Err, got Ok({:?})",
            result.as_ref().ok()
        );
    }
}

#[track_caller]
pub fn equal_error_display(err: &dyn Error, expected: &str) {
    let s = err.to_string();
    if s != expected {
        panic!(
            "assertion failed: `equal_error_display`\n  expected: `{expected}`\n    actual: `{s}`"
        );
    }
}

#[track_caller]
pub fn error_contains(err: &dyn Error, needle: &str) {
    let s = err.to_string();
    if !s.contains(needle) {
        panic!("assertion failed: `error_contains`\n  needle: `{needle}`\n  display: `{s}`");
    }
}

#[track_caller]
pub fn error_chain_contains(err: &dyn Error, needle: &str) {
    let mut buf = err.to_string();
    let mut s = err.source();
    while let Some(e) = s {
        buf.push_str(": ");
        buf.push_str(&e.to_string());
        s = e.source();
    }
    if !buf.contains(needle) {
        panic!("assertion failed: `error_chain_contains`\n  needle: `{needle}`\n  chain: `{buf}`");
    }
}

#[track_caller]
pub fn error_is<E: Error + PartialEq + 'static>(err: &(dyn Error + 'static), target: &E) {
    let mut e: Option<&dyn Error> = Some(err);
    while let Some(cur) = e {
        if let Some(x) = cur.downcast_ref::<E>()
            && x == target
        {
            return;
        }
        e = cur.source();
    }
    panic!(
        "assertion failed: `error_is`: chain does not contain matching `{:?}`",
        target
    );
}

#[track_caller]
pub fn not_error_is<E: Error + PartialEq + 'static>(err: &(dyn Error + 'static), target: &E) {
    let mut e: Option<&dyn Error> = Some(err);
    while let Some(cur) = e {
        if let Some(x) = cur.downcast_ref::<E>()
            && x == target
        {
            panic!(
                "assertion failed: `not_error_is`: chain unexpectedly contains `{:?}`",
                target
            );
        }
        e = cur.source();
    }
}

#[track_caller]
pub fn error_as<'a, T: Error + 'static>(err: &'a (dyn Error + 'static)) -> &'a T {
    err.downcast_ref::<T>().unwrap_or_else(|| {
        panic!(
            "assertion failed: `error_as`: could not downcast to `{}`",
            std::any::type_name::<T>()
        )
    })
}

#[track_caller]
pub fn error_as_chain<'a, T: Error + 'static>(err: &'a (dyn Error + 'static)) -> &'a T {
    let mut e: Option<&dyn Error> = Some(err);
    while let Some(cur) = e {
        if let Some(x) = cur.downcast_ref::<T>() {
            return x;
        }
        e = cur.source();
    }
    panic!(
        "assertion failed: `error_as_chain`: could not downcast to `{}`",
        std::any::type_name::<T>()
    );
}

#[track_caller]
pub fn not_error_as<T: Error + 'static>(err: &(dyn Error + 'static)) {
    let mut e: Option<&dyn Error> = Some(err);
    while let Some(cur) = e {
        if cur.downcast_ref::<T>().is_some() {
            panic!(
                "assertion failed: `not_error_as`: unexpectedly `{}`",
                std::any::type_name::<T>()
            );
        }
        e = cur.source();
    }
}

#[track_caller]
pub fn io_error_kind(err: &std::io::Error, expected: std::io::ErrorKind) {
    if err.kind() != expected {
        panic!(
            "assertion failed: `io_error_kind`\n  expected: `{expected:?}`\n    actual: `{:?}`",
            err.kind()
        );
    }
}
