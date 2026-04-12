mod arguments;
mod matchers;

pub use arguments::{Arguments, TestingT};
pub use matchers::{anything, anything_of_type, eq, matched_by};

use std::any::Any;
use std::sync::Mutex;
use std::time::Duration;

use arguments::Arguments as Args;
use matchers::Matcher;

type RunHook = Box<dyn Fn(&Args) + Send + 'static>;

pub struct Mock {
    inner: Mutex<MockInner>,
}

struct MockInner {
    expectations: Vec<Expectation>,
    calls: Vec<RecordedCall>,
    next_id: usize,
}

struct RecordedCall {
    method: &'static str,
    args: Vec<Box<dyn Any + Send>>,
}

struct Expectation {
    id: usize,
    method: &'static str,
    matchers: Vec<Box<dyn Matcher>>,
    return_gen: ReturnGen,
    remaining: Option<u32>,
    maybe: bool,
    removed: bool,
    run: Option<RunHook>,
    delay: Option<Duration>,
    panic_msg: Option<String>,
    invocations: u32,
    assert_exact: Option<u32>,
}

enum ReturnGen {
    Fn(Box<dyn Fn() -> Vec<Box<dyn Any + Send>> + Send + 'static>),
}

pub struct CallBuilder<'a> {
    mock: &'a Mock,
    method: &'static str,
    matchers: Vec<Box<dyn Matcher>>,
}

pub struct PostReturn<'a, F>
where
    F: Fn() -> Vec<Box<dyn Any + Send>> + Send + 'static,
{
    mock: &'a Mock,
    method: &'static str,
    matchers: Vec<Box<dyn Matcher>>,
    f: F,
    run: Option<RunHook>,
    delay: Option<Duration>,
    panic_msg: Option<String>,
    maybe: bool,
    remaining: Option<u32>,
    assert_exact: Option<u32>,
}

pub struct ActiveExpectation<'a> {
    mock: &'a Mock,
    id: usize,
}

impl<'a> CallBuilder<'a> {
    pub fn returning<F>(self, f: F) -> PostReturn<'a, F>
    where
        F: Fn() -> Vec<Box<dyn Any + Send>> + Send + 'static,
    {
        PostReturn {
            mock: self.mock,
            method: self.method,
            matchers: self.matchers,
            f,
            run: None,
            delay: None,
            panic_msg: None,
            maybe: false,
            remaining: None,
            assert_exact: None,
        }
    }
}

impl<'a, F> PostReturn<'a, F>
where
    F: Fn() -> Vec<Box<dyn Any + Send>> + Send + 'static,
{
    pub fn run<R: Fn(&Args) + Send + 'static>(mut self, r: R) -> Self {
        self.run = Some(Box::new(r));
        self
    }

    pub fn after(mut self, d: Duration) -> Self {
        self.delay = Some(d);
        self
    }

    pub fn maybe(mut self) -> ActiveExpectation<'a> {
        self.maybe = true;
        self.remaining = None;
        self.assert_exact = None;
        self.commit()
    }

    pub fn once(mut self) -> ActiveExpectation<'a> {
        self.remaining = Some(1);
        self.assert_exact = Some(1);
        self.commit()
    }

    pub fn twice(mut self) -> ActiveExpectation<'a> {
        self.remaining = Some(2);
        self.assert_exact = Some(2);
        self.commit()
    }

    pub fn times(mut self, n: u32) -> ActiveExpectation<'a> {
        self.remaining = Some(n);
        self.assert_exact = Some(n);
        self.commit()
    }

    pub fn unlimited(mut self) -> ActiveExpectation<'a> {
        self.remaining = None;
        self.assert_exact = None;
        self.commit()
    }

    pub fn finish(self) -> ActiveExpectation<'a> {
        self.unlimited()
    }

    pub fn panic(mut self, msg: impl Into<String>) -> ActiveExpectation<'a> {
        self.panic_msg = Some(msg.into());
        self.remaining = Some(1);
        self.assert_exact = Some(1);
        self.commit()
    }

    fn commit(self) -> ActiveExpectation<'a> {
        let mut inner = self.mock.inner.lock().unwrap();
        let id = inner.next_id;
        inner.next_id += 1;
        let f = self.f;
        inner.expectations.push(Expectation {
            id,
            method: self.method,
            matchers: self.matchers,
            return_gen: ReturnGen::Fn(Box::new(f)),
            remaining: self.remaining,
            maybe: self.maybe,
            removed: false,
            run: self.run,
            delay: self.delay,
            panic_msg: self.panic_msg,
            invocations: 0,
            assert_exact: self.assert_exact,
        });
        ActiveExpectation {
            mock: self.mock,
            id,
        }
    }
}

impl ActiveExpectation<'_> {
    pub fn unset(self) {
        let mut inner = self.mock.inner.lock().unwrap();
        for e in &mut inner.expectations {
            if e.id == self.id {
                e.removed = true;
                break;
            }
        }
    }
}

impl Default for Mock {
    fn default() -> Self {
        Self::new()
    }
}

impl Mock {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(MockInner {
                expectations: Vec::new(),
                calls: Vec::new(),
                next_id: 0,
            }),
        }
    }

    pub fn on<'a>(
        &'a self,
        method: &'static str,
        matchers: Vec<Box<dyn Matcher>>,
    ) -> CallBuilder<'a> {
        CallBuilder {
            mock: self,
            method,
            matchers,
        }
    }

    pub fn method_called(&self, method: &'static str, args: Args) -> Args {
        let (sleep, panic_m, out) = {
            let mut inner = self.inner.lock().unwrap();
            let mut matched_idx = None;
            for (i, exp) in inner.expectations.iter().enumerate() {
                if exp.removed || exp.method != method {
                    continue;
                }
                let can = exp.remaining.is_none_or(|n| n > 0);
                let ok = can && args_match(&exp.matchers, args.as_raw());
                if ok {
                    matched_idx = Some(i);
                    break;
                }
            }
            let Some(i) = matched_idx else {
                drop(inner);
                panic!("unexpected call to {method}");
            };
            let (sleep, panic_m, out) = {
                let exp = &mut inner.expectations[i];
                if let Some(r) = &exp.run {
                    r(&args);
                }
                exp.invocations += 1;
                match exp.remaining {
                    None => {}
                    Some(1) => {
                        exp.remaining = Some(0);
                    }
                    Some(n) => {
                        exp.remaining = Some(n - 1);
                    }
                }
                let sleep = exp.delay;
                let panic_m = exp.panic_msg.clone();
                let ReturnGen::Fn(ret_fn) = &exp.return_gen;
                let out = ret_fn();
                (sleep, panic_m, out)
            };
            let boxes = args.into_boxes();
            inner.calls.push(RecordedCall {
                method,
                args: boxes,
            });
            (sleep, panic_m, out)
        };
        if let Some(d) = sleep {
            std::thread::sleep(d);
        }
        if let Some(msg) = panic_m {
            panic!("{msg}");
        }
        Args::from_boxes(out)
    }

    pub fn assert_expectations(&self, t: &dyn TestingT) -> bool {
        let inner = self.inner.lock().unwrap();
        for exp in &inner.expectations {
            if exp.removed || exp.maybe {
                continue;
            }
            let ok = match exp.assert_exact {
                None => exp.invocations >= 1,
                Some(n) => exp.invocations == n,
            };
            if !ok {
                t.errorf(&format!(
                    "expectation for {} not met: got {} invocations",
                    exp.method, exp.invocations
                ));
                t.fail_now();
                return false;
            }
        }
        true
    }

    pub fn assert_called(
        &self,
        t: &dyn TestingT,
        method: &'static str,
        matchers: &[Box<dyn Matcher>],
    ) -> bool {
        let inner = self.inner.lock().unwrap();
        for rec in &inner.calls {
            if rec.method != method {
                continue;
            }
            if args_match(matchers, &rec.args) {
                return true;
            }
        }
        t.errorf(&format!(
            "expected {method} to have been called with matching arguments"
        ));
        t.fail_now();
        false
    }

    pub fn assert_not_called(
        &self,
        t: &dyn TestingT,
        method: &'static str,
        matchers: &[Box<dyn Matcher>],
    ) -> bool {
        let inner = self.inner.lock().unwrap();
        for rec in &inner.calls {
            if rec.method != method {
                continue;
            }
            if args_match(matchers, &rec.args) {
                t.errorf(&format!(
                    "expected {method} not to have been called with matching arguments"
                ));
                t.fail_now();
                return false;
            }
        }
        true
    }

    pub fn assert_number_of_calls(&self, t: &dyn TestingT, method: &'static str, n: usize) -> bool {
        let inner = self.inner.lock().unwrap();
        let c = inner.calls.iter().filter(|r| r.method == method).count();
        if c != n {
            t.errorf(&format!("expected {n} calls to {method}, got {c}"));
            t.fail_now();
            return false;
        }
        true
    }
}

pub fn assert_expectations_for_objects(t: &dyn TestingT, mocks: &[&Mock]) -> bool {
    for m in mocks {
        if !m.assert_expectations(t) {
            return false;
        }
    }
    true
}

fn args_match(matchers: &[Box<dyn Matcher>], args: &[Box<dyn Any + Send>]) -> bool {
    if matchers.len() != args.len() {
        return false;
    }
    matchers
        .iter()
        .zip(args.iter())
        .all(|(m, a)| m.matches(a.as_ref()))
}

#[macro_export]
macro_rules! mock_args {
    () => {
        $crate::mock::Arguments::from_boxes(::std::vec::Vec::new())
    };
    ($($x:expr),+ $(,)?) => {{
        let mut v = ::std::vec::Vec::new();
        $(
            v.push(
                ::std::boxed::Box::new($x) as ::std::boxed::Box<dyn ::std::any::Any + Send>
            );
        )*
        $crate::mock::Arguments::from_boxes(v)
    }};
}

#[cfg(test)]
mod mock_test;
