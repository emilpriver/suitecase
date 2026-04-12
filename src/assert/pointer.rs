use std::sync::{Arc, Weak};

#[track_caller]
pub fn same_ref<T: ?Sized>(a: &T, b: &T) {
    if !std::ptr::eq(a, b) {
        panic!("assertion failed: `same_ref`: references do not point to the same allocation");
    }
}

#[track_caller]
pub fn not_same_ref<T: ?Sized>(a: &T, b: &T) {
    if std::ptr::eq(a, b) {
        panic!("assertion failed: `not_same_ref`: references point to the same allocation");
    }
}

#[track_caller]
pub fn same_arc<T: ?Sized>(a: &Arc<T>, b: &Arc<T>) {
    if !Arc::ptr_eq(a, b) {
        panic!("assertion failed: `same_arc`: Arc pointers differ");
    }
}

#[track_caller]
pub fn same_weak<T: ?Sized>(a: &Weak<T>, b: &Weak<T>) {
    if !Weak::ptr_eq(a, b) {
        panic!("assertion failed: `same_weak`: Weak pointers differ");
    }
}
