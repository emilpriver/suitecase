use std::sync::Arc;

use super::pointer::*;

#[test]
fn same_ref_ok() {
    let v = 5_i32;
    same_ref(&v, &v);
}

#[test]
fn not_same_ref_ok() {
    let a = 1_i32;
    let b = 1_i32;
    not_same_ref(&a, &b);
}

#[test]
fn same_arc_ok() {
    let a = Arc::new(1);
    let b = Arc::clone(&a);
    same_arc(&a, &b);
}

#[test]
fn same_weak_ok() {
    let a: Arc<i32> = Arc::new(7);
    let w1 = Arc::downgrade(&a);
    let w2 = Arc::downgrade(&a);
    same_weak(&w1, &w2);
}
