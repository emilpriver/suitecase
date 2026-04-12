use std::time::Duration;

use super::time::*;

#[test]
fn within_duration_ok() {
    within_duration(
        Duration::from_secs(10),
        Duration::from_secs(10),
        Duration::from_millis(1),
    );
}

#[test]
fn within_range_ok() {
    let mid = Duration::from_secs(5);
    let start = Duration::from_secs(1);
    let end = Duration::from_secs(10);
    within_range(mid, start, end);
}
