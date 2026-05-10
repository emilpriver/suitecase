use std::panic::panic_any;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FailReason {
    #[error("Fail: {0}")]
    Fail(String),

    #[error("FailNow: {0}")]
    FailNow(String),
}

/// Mark the current case as failed; remaining cases continue.
///
/// # Panics
///
/// Always panics with a [`FailReason::Fail`]. The runner catches this and records
/// the case as failed without re-panicking.
#[track_caller]
pub fn fail(reason: &str) -> ! {
    panic_any(FailReason::Fail(reason.to_string()))
}

/// Mark the current case as failed and abort all remaining cases.
///
/// The runner catches this, records the case as failed, runs `teardown_suite`
/// if present, then panics with the message.
///
/// # Panics
///
/// Always panics with a [`FailReason::FailNow`].
#[track_caller]
pub fn fail_now(reason: &str) -> ! {
    panic_any(FailReason::FailNow(reason.to_string()))
}
