use std::any::Any;
use std::fmt;

use super::matchers::Matcher;

pub struct Arguments {
    values: Vec<Box<dyn Any + Send>>,
}

impl Arguments {
    pub fn from_boxes(values: Vec<Box<dyn Any + Send>>) -> Self {
        Self { values }
    }

    pub fn into_boxes(self) -> Vec<Box<dyn Any + Send>> {
        self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn as_raw(&self) -> &[Box<dyn Any + Send>] {
        &self.values
    }

    pub fn get<T: 'static>(&self, index: usize) -> Option<&T> {
        self.values.get(index).and_then(|b| b.downcast_ref())
    }

    pub fn get_box(&self, index: usize) -> Option<&(dyn Any + Send)> {
        self.values.get(index).map(|b| b.as_ref())
    }

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
                } else if let Some(v) = b.downcast_ref::<u64>() {
                    Some(*v as i64)
                } else {
                    None
                }
            })
            .expect("argument is not an integer")
    }

    pub fn bool(&self, index: usize) -> bool {
        *self
            .values
            .get(index)
            .and_then(|b| b.downcast_ref::<bool>())
            .expect("argument is not bool")
    }

    pub fn string(&self, index: usize) -> String {
        if let Some(s) = self.get::<String>(index) {
            return s.clone();
        }
        self.get::<&'static str>(index)
            .map(|s| (*s).to_string())
            .expect("argument is not a string")
    }

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

pub trait TestingT {
    fn errorf(&self, msg: &str);
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
