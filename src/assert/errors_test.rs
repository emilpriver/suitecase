use std::error::Error;
use std::fmt;
use std::io;

use super::errors::*;

#[derive(Debug, PartialEq, Eq)]
struct CustomErr(&'static str);

impl fmt::Display for CustomErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl Error for CustomErr {}

#[test]
fn is_error_ok() {
    is_error(&Err::<i32, &str>("e"));
}

#[test]
#[should_panic(expected = "`is_error`")]
fn is_error_fails() {
    is_error(&Ok::<i32, &str>(1));
}

#[test]
fn equal_error_display_ok() {
    let e = io::Error::other("msg");
    equal_error_display(&e, "msg");
}

#[test]
fn error_contains_ok() {
    let e = io::Error::other("hello world");
    error_contains(&e, "world");
}

#[derive(Debug)]
struct WithSource(io::Error);

impl fmt::Display for WithSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "outer: {}", self.0)
    }
}

impl Error for WithSource {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

#[test]
fn error_chain_contains_ok() {
    let inner = io::Error::other("inner_token");
    let err = WithSource(inner);
    error_chain_contains(&err, "inner_token");
}

#[test]
fn error_is_ok() {
    let target = CustomErr("match");
    let err = CustomErr("match");
    let er: &(dyn Error + 'static) = &err;
    error_is(er, &target);
}

#[test]
#[should_panic(expected = "`error_is`")]
fn error_is_fails() {
    let target = CustomErr("a");
    let err = CustomErr("b");
    let er: &(dyn Error + 'static) = &err;
    error_is(er, &target);
}

#[test]
fn not_error_is_ok() {
    let target = CustomErr("want");
    let err = CustomErr("other");
    let er: &(dyn Error + 'static) = &err;
    not_error_is(er, &target);
}

#[test]
#[should_panic(expected = "`not_error_is`")]
fn not_error_is_fails() {
    let target = CustomErr("same");
    let err = CustomErr("same");
    let er: &(dyn Error + 'static) = &err;
    not_error_is(er, &target);
}

#[test]
fn error_as_ok() {
    let e = io::Error::other("z");
    let er: &(dyn Error + 'static) = &e;
    let x: &io::Error = error_as(er);
    assert_eq!(x.to_string(), "z");
}

#[derive(Debug)]
struct Ghost;

impl fmt::Display for Ghost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ghost")
    }
}

impl Error for Ghost {}

#[test]
fn error_as_chain_finds_inner() {
    let inner = io::Error::other("inner");
    let wrap = WithSource(inner);
    let er: &(dyn Error + 'static) = &wrap;
    let io_ref: &io::Error = error_as_chain(er);
    assert!(io_ref.to_string().contains("inner"));
}

#[test]
fn not_error_as_ok() {
    let e = io::Error::other("z");
    let er: &(dyn Error + 'static) = &e;
    not_error_as::<Ghost>(er);
}

#[test]
#[should_panic(expected = "`not_error_as`")]
fn not_error_as_fails() {
    let e = io::Error::other("z");
    let er: &(dyn Error + 'static) = &e;
    not_error_as::<io::Error>(er);
}

#[test]
fn io_error_kind_ok() {
    let e = io::Error::from(io::ErrorKind::NotFound);
    io_error_kind(&e, io::ErrorKind::NotFound);
}
