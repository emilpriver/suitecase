//! Assertions for [`std::error::Error`] values and chains.

use std::error::Error;
use std::fmt::Debug;

/// Asserts that `result` is `Err`.
///
/// # Panics
///
/// Panics when `result` is `Ok`.
///
/// # Examples
///
/// ```
/// use suitecase::assert::is_error;
///
/// let r: Result<(), &str> = Err("e");
/// is_error(&r);
/// ```
#[track_caller]
pub fn is_error<T: Debug, E>(result: &Result<T, E>) {
    if result.is_ok() {
        panic!(
            "assertion failed: `is_error`: expected Err, got Ok({:?})",
            result.as_ref().ok()
        );
    }
}

/// Asserts that `err.to_string()` equals `expected` exactly.
///
/// # Panics
///
/// Panics when the display strings differ.
///
/// # Examples
///
/// ```
/// use std::io;
/// use suitecase::assert::equal_error_display;
///
/// let err = io::Error::new(io::ErrorKind::NotFound, "missing");
/// equal_error_display(&err, "missing");
/// ```
#[track_caller]
pub fn equal_error_display(err: &dyn Error, expected: &str) {
    let s = err.to_string();
    if s != expected {
        panic!(
            "assertion failed: `equal_error_display`\n  expected: `{expected}`\n    actual: `{s}`"
        );
    }
}

/// Asserts that `err.to_string()` contains `needle`.
///
/// # Panics
///
/// Panics when the substring is not found.
///
/// # Examples
///
/// ```
/// use std::io;
/// use suitecase::assert::error_contains;
///
/// let err = io::Error::new(io::ErrorKind::Other, "oops: file");
/// error_contains(&err, "file");
/// ```
#[track_caller]
pub fn error_contains(err: &dyn Error, needle: &str) {
    let s = err.to_string();
    if !s.contains(needle) {
        panic!("assertion failed: `error_contains`\n  needle: `{needle}`\n  display: `{s}`");
    }
}

/// Asserts that the full error chain’s concatenated display text contains `needle`.
///
/// # Panics
///
/// Panics when `needle` is not found in the chain text.
///
/// # Examples
///
/// ```
/// use std::io;
/// use suitecase::assert::error_chain_contains;
///
/// let inner = io::Error::new(io::ErrorKind::Other, "inner");
/// let err = io::Error::new(io::ErrorKind::Other, inner);
/// error_chain_contains(&err, "inner");
/// ```
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

/// Walks the error chain and asserts some link equals `target` (via `PartialEq`).
///
/// # Panics
///
/// Panics when no matching error is found in the chain.
///
/// # Examples
///
/// ```
/// use std::error::Error;
/// use std::fmt;
/// use suitecase::assert::error_is;
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct E(&'static str);
/// impl fmt::Display for E {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         f.write_str(self.0)
///     }
/// }
/// impl Error for E {}
///
/// let target = E("match");
/// let err = E("match");
/// let er: &(dyn Error + 'static) = &err;
/// error_is(er, &target);
/// ```
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

/// Asserts that no link in the error chain equals `target`.
///
/// # Panics
///
/// Panics when a matching error appears in the chain.
///
/// # Examples
///
/// ```
/// use std::error::Error;
/// use std::fmt;
/// use suitecase::assert::not_error_is;
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct E(&'static str);
/// impl fmt::Display for E {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         f.write_str(self.0)
///     }
/// }
/// impl Error for E {}
///
/// let target = E("want");
/// let err = E("other");
/// let er: &(dyn Error + 'static) = &err;
/// not_error_is(er, &target);
/// ```
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

/// Downcasts `err` to `T` or panics.
///
/// # Panics
///
/// Panics when `err` is not `T`.
///
/// # Examples
///
/// ```
/// use std::io;
/// use suitecase::assert::error_as;
///
/// let err: &(dyn std::error::Error + 'static) = &io::Error::new(io::ErrorKind::Other, "x");
/// let io_err: &io::Error = error_as(err);
/// assert_eq!(io_err.kind(), io::ErrorKind::Other);
/// ```
#[track_caller]
pub fn error_as<'a, T: Error + 'static>(err: &'a (dyn Error + 'static)) -> &'a T {
    err.downcast_ref::<T>().unwrap_or_else(|| {
        panic!(
            "assertion failed: `error_as`: could not downcast to `{}`",
            std::any::type_name::<T>()
        )
    })
}

/// Finds the first `T` in the error chain via downcast.
///
/// # Panics
///
/// Panics when no link downcasts to `T`.
///
/// # Examples
///
/// ```
/// use std::io;
/// use suitecase::assert::error_as_chain;
///
/// let inner = io::Error::new(io::ErrorKind::Other, "inner");
/// let err: &(dyn std::error::Error + 'static) = &io::Error::new(io::ErrorKind::Other, inner);
/// let inner_io: &io::Error = error_as_chain(err);
/// assert_eq!(inner_io.to_string(), "inner");
/// ```
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

/// Asserts that no link in the chain downcasts to `T`.
///
/// # Panics
///
/// Panics when some link is a `T`.
///
/// # Examples
///
/// ```
/// use std::io;
/// use suitecase::assert::not_error_as;
///
/// let err: &(dyn std::error::Error + 'static) = &io::Error::new(io::ErrorKind::Other, "x");
/// not_error_as::<std::fmt::Error>(err);
/// ```
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

/// Asserts that `err.kind()` equals `expected`.
///
/// # Panics
///
/// Panics when the kinds differ.
///
/// # Examples
///
/// ```
/// use std::io;
/// use suitecase::assert::io_error_kind;
///
/// let err = io::Error::new(io::ErrorKind::NotFound, "nope");
/// io_error_kind(&err, io::ErrorKind::NotFound);
/// ```
#[track_caller]
pub fn io_error_kind(err: &std::io::Error, expected: std::io::ErrorKind) {
    if err.kind() != expected {
        panic!(
            "assertion failed: `io_error_kind`\n  expected: `{expected:?}`\n    actual: `{:?}`",
            err.kind()
        );
    }
}
