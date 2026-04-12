//! Pointer and reference identity checks ([`Arc`], [`Weak`]).

use std::sync::{Arc, Weak};

/// Asserts that `a` and `b` refer to the same memory location ([`std::ptr::eq`]).
///
/// # Panics
///
/// Panics when the addresses differ.
///
/// # Examples
///
/// ```
/// use suitecase::assert::same_ref;
///
/// let v = 5_i32;
/// same_ref(&v, &v);
/// ```
#[track_caller]
pub fn same_ref<T: ?Sized>(a: &T, b: &T) {
    if !std::ptr::eq(a, b) {
        panic!("assertion failed: `same_ref`: references do not point to the same allocation");
    }
}

/// Asserts that `a` and `b` are distinct references ([`std::ptr::eq`] is false).
///
/// # Panics
///
/// Panics when the addresses are equal.
///
/// # Examples
///
/// ```
/// use suitecase::assert::not_same_ref;
///
/// let a = 1_i32;
/// let b = 1_i32;
/// not_same_ref(&a, &b);
/// ```
#[track_caller]
pub fn not_same_ref<T: ?Sized>(a: &T, b: &T) {
    if std::ptr::eq(a, b) {
        panic!("assertion failed: `not_same_ref`: references point to the same allocation");
    }
}

/// Asserts that two [`Arc`] pointers reference the same allocation.
///
/// # Panics
///
/// Panics when [`Arc::ptr_eq`] is false.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use suitecase::assert::same_arc;
///
/// let a = Arc::new(1);
/// let b = Arc::clone(&a);
/// same_arc(&a, &b);
/// ```
#[track_caller]
pub fn same_arc<T: ?Sized>(a: &Arc<T>, b: &Arc<T>) {
    if !Arc::ptr_eq(a, b) {
        panic!("assertion failed: `same_arc`: Arc pointers differ");
    }
}

/// Asserts that two [`Weak`] pointers refer to the same allocation.
///
/// # Panics
///
/// Panics when [`Weak::ptr_eq`] is false.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use suitecase::assert::same_weak;
///
/// let a: Arc<i32> = Arc::new(7);
/// let w1 = Arc::downgrade(&a);
/// let w2 = Arc::downgrade(&a);
/// same_weak(&w1, &w2);
/// ```
#[track_caller]
pub fn same_weak<T: ?Sized>(a: &Weak<T>, b: &Weak<T>) {
    if !Weak::ptr_eq(a, b) {
        panic!("assertion failed: `same_weak`: Weak pointers differ");
    }
}
