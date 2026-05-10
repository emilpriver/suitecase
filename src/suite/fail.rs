use thiserror::Error;

#[derive(Error, Debug)]
pub enum FailReason {
    #[error("Fail: {0}")]
    Fail(String),

    #[error("Fail: {0}")]
    FailNow(String),
}

pub fn fail(reason: String) -> FailReason {
    FailReason::Fail(reason)
}

pub fn fail_now(reason: String) -> FailReason {
    FailReason::FailNow(reason)
}
