//! Typed access to values recorded or returned through [`Mock::method_called`](crate::mock::Mock::method_called).
//!
//! Values are stored as [`Box<dyn Any + Send>`](std::any::Any); use [`Arguments::get`] or the
//! helpers [`Arguments::int`], [`Arguments::bool`], [`Arguments::string`], etc.

use std::any::Any;
use std::fmt;

use super::matchers::Matcher;

/// A list of boxed arguments or return values for a single mock call.
///
/// # Examples
///
/// ```
/// use suitecase::mock::Arguments;
///
/// let a = Arguments::from_boxes(vec![Box::new(40i32), Box::new(2i32)]);
/// assert_eq!(a.int(0) + a.int(1), 42);
/// ```
pub struct Arguments {
    values: Vec<Box<dyn Any + Send>>,
}

impl Arguments {
    /// Wraps a vector of boxed values (one per positional argument or return slot).
    pub fn from_boxes(values: Vec<Box<dyn Any + Send>>) -> Self {
        Self { values }
    }

    /// Consumes `self` and returns the underlying boxes.
    pub fn into_boxes(self) -> Vec<Box<dyn Any + Send>> {
        self.values
    }

    /// Number of slots.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// `true` when there are no slots.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Raw slice of argument boxes.
    pub fn as_raw(&self) -> &[Box<dyn Any + Send>] {
        &self.values
    }

    /// Returns a reference to slot `index` if its type matches `T`.
    pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
        self.values.get(index).and_then(|b| b.downcast_ref())
    }

    /// Like [`get`](Self::get) but returns the trait object for custom downcasting.
    pub fn get_box(&self, index: usize) -> Option<&(dyn Any + Send)> {
        self.values.get(index).map(|b| b.as_ref())
    }

    /// Reads an integer slot, accepting several fixed-width integer types.
    ///
    /// # Panics
    ///
    /// Panics if the slot is missing or not an integer type supported by this helper.
    pub fn int(&self, index: usize) -> i64 {
        self.values
            .get(index)
            .and_then(|b| {
                if let Some(v) = b.downcast_ref::<i64>() {
                    Some(*v)
                } else if let Some(v) = b.downcast_ref::<i32>() {
                    Some(*v as i64)
                } else if let Some(v) = b.downcast_ref::<isize>() {
                    Some(*v as i64)
                } else if let Some(v) = b.downcast_ref::<u32>() {
                    Some(*v as i64)
                } else {
                    b.downcast_ref::<u64>().map(|v| *v as i64)
                }
            })
            .expect("argument is not an integer")
    }

    /// Reads a `bool` slot.
    ///
    /// # Panics
    ///
    /// Panics if the slot is missing or not a `bool`.
    pub fn bool(&self, index: usize) -> bool {
        *self
            .values
            .get(index)
            .and_then(|b| b.downcast_ref::<bool>())
            .expect("argument is not bool")
    }

    /// Reads a [`String`] or `&'static str` slot.
    ///
    /// # Panics
    ///
    /// Panics if the slot is missing or not string-like as above.
    pub fn string(&self, index: usize) -> String {
        if let Some(s) = self.get::<String>(index) {
            return s.clone();
        }
        self.get::<&'static str>(index)
            .map(|s| (*s).to_string())
            .expect("argument is not a string")
    }

    /// Best-effort string for error-like values (`String`, `&str`, otherwise [`TypeId`](std::any::TypeId) debug).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of range.
    pub fn error_display(&self, index: usize) -> String {
        let b = self.values.get(index).expect("missing argument");
        if let Some(e) = b.downcast_ref::<String>() {
            return e.clone();
        }
        if let Some(e) = b.downcast_ref::<&'static str>() {
            return (*e).to_string();
        }
        format!("{}", MockErrorDebug(b.as_ref()))
    }

    /// Compares this argument list against `expected` matchers, notifying `testing` on failure.
    ///
    /// Returns `false` after the first failed check.
    ///
    /// # Examples
    ///
    /// ```
    /// use suitecase::mock::{eq, Arguments, TestingT};
    ///
    /// struct T;
    /// impl TestingT for T {
    ///     fn errorf(&self, _: &str) {}
    ///     fn fail_now(&self) {}
    /// }
    ///
    /// let a = Arguments::from_boxes(vec![Box::new(7i32)]);
    /// assert!(a.assert_matches(&T, &[eq(7i32)]));
    /// ```
    pub fn assert_matches(&self, testing: &dyn TestingT, expected: &[Box<dyn Matcher>]) -> bool {
        if self.values.len() != expected.len() {
            testing.errorf(&format!(
                "argument count mismatch: got {} want {}",
                self.values.len(),
                expected.len()
            ));
            testing.fail_now();
            return false;
        }
        for (i, (arg, m)) in self.values.iter().zip(expected.iter()).enumerate() {
            if !m.matches(arg.as_ref()) {
                testing.errorf(&format!("argument {i} did not match"));
                testing.fail_now();
                return false;
            }
        }
        true
    }
}

struct MockErrorDebug<'a>(&'a dyn Any);

impl fmt::Display for MockErrorDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(e) = self.0.downcast_ref::<String>() {
            return write!(f, "{e}");
        }
        if let Some(e) = self.0.downcast_ref::<&'static str>() {
            return write!(f, "{e}");
        }
        write!(f, "{:?}", self.0.type_id())
    }
}

/// Hooks for reporting assertion failures from mock verification (similar to Go’s `testing.T`).
///
/// Implement this for your test context: typically [`errorf`](TestingT::errorf) records a message
/// and [`fail_now`](TestingT::fail_now) marks the test as failed or aborts the current check.
///
/// # Examples
///
/// ```
/// use suitecase::mock::TestingT;
///
/// struct Counting;
///
/// impl TestingT for Counting {
///     fn errorf(&self, _: &str) {}
///     fn fail_now(&self) {}
/// }
/// ```
pub trait TestingT {
    /// Records a failure message (e.g. log or buffer).
    fn errorf(&self, msg: &str);
    /// Marks the current test or assertion as failed (for example by panicking or setting a flag).
    fn fail_now(&self);
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::super::eq;
    use super::*;

    struct RecordingT {
        messages: Mutex<Vec<String>>,
        fail_now_hits: AtomicUsize,
    }

    impl RecordingT {
        fn new() -> Self {
            Self {
                messages: Mutex::new(Vec::new()),
                fail_now_hits: AtomicUsize::new(0),
            }
        }
    }

    impl TestingT for RecordingT {
        fn errorf(&self, msg: &str) {
            self.messages.lock().unwrap().push(msg.to_string());
        }

        fn fail_now(&self) {
            self.fail_now_hits.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn int_coercion() {
        let a = Arguments::from_boxes(vec![Box::new(42i32)]);
        assert_eq!(a.int(0), 42);
    }

    #[test]
    fn assert_matches_succeeds_when_matchers_align() {
        let a = Arguments::from_boxes(vec![Box::new(1i32), Box::new("z".to_string())]);
        let t = RecordingT::new();
        assert!(a.assert_matches(&t, &[eq(1i32), eq("z".to_string())]));
        assert_eq!(t.fail_now_hits.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn assert_matches_fails_on_len_mismatch() {
        let a = Arguments::from_boxes(vec![Box::new(1i32)]);
        let t = RecordingT::new();
        assert!(!a.assert_matches(&t, &[eq(1i32), eq(2i32)]));
        assert_eq!(t.fail_now_hits.load(Ordering::SeqCst), 1);
        assert!(
            t.messages.lock().unwrap()[0].contains("argument count mismatch"),
            "{:?}",
            t.messages.lock().unwrap()
        );
    }

    #[test]
    fn assert_matches_fails_on_predicate() {
        let a = Arguments::from_boxes(vec![Box::new(9i32)]);
        let t = RecordingT::new();
        assert!(!a.assert_matches(&t, &[eq(1i32)]));
        assert_eq!(t.fail_now_hits.load(Ordering::SeqCst), 1);
    }
}
