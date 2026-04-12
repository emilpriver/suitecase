use std::panic::{AssertUnwindSafe, catch_unwind};

#[track_caller]
pub fn panics(f: impl FnOnce()) {
    let r = catch_unwind(AssertUnwindSafe(f));
    if r.is_ok() {
        panic!("assertion failed: `panics`: expected closure to panic");
    }
}

#[track_caller]
pub fn not_panics(f: impl FnOnce()) {
    let r = catch_unwind(AssertUnwindSafe(f));
    if let Err(payload) = r {
        panic!(
            "assertion failed: `not_panics`: closure panicked: {:?}",
            panic_payload_to_string(payload)
        );
    }
}

fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<&'static str>()
        .map(|s| (*s).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "(opaque panic payload)".to_string())
}

#[track_caller]
pub fn panics_with_substring(f: impl FnOnce(), needle: &str) {
    let r = catch_unwind(AssertUnwindSafe(f));
    match r {
        Ok(()) => panic!("assertion failed: `panics_with_substring`: expected panic"),
        Err(payload) => {
            let s = panic_payload_to_string(payload);
            if !s.contains(needle) {
                panic!(
                    "assertion failed: `panics_with_substring`\n  needle: `{needle}`\n message: `{s}`"
                );
            }
        }
    }
}
