//! Test doubles with **expectations** and **call recording**, in the spirit of Go’s
//! [testify/mock](https://pkg.go.dev/github.com/stretchr/testify/mock).
//!
//! # Model
//!
//! 1. Create a [`Mock`] and register expectations with [`Mock::on`] → [`CallBuilder::returning`],
//!    then finalize with [`PostReturn::finish`], [`PostReturn::once`], [`PostReturn::maybe`], etc.
//! 2. From your test double, invoke [`Mock::method_called`] with a logical method name and
//!    [`Arguments`] built with [`mock_args!`] ([`crate::mock_args`]) or [`Arguments::from_boxes`].
//! 3. Read return values from the returned [`Arguments`] (e.g. [`Arguments::string`],
//!    [`Arguments::int`]).
//! 4. Verify with [`Mock::assert_expectations`] and related helpers, passing a [`TestingT`]
//!    implementation that forwards failures to your test output.
//!
//! Unexpected calls **panic** from [`Mock::method_called`]. Failed assertions return `false` and
//! invoke [`TestingT::errorf`] / [`TestingT::fail_now`].
//!
//! # Imports
//!
//! ```no_run
//! use suitecase::mock::{eq, Mock, TestingT};
//! use suitecase::mock_args;
//! ```
//!
//! # Example
//!
//! ```
//! use suitecase::mock::{eq, Mock, TestingT};
//!
//! struct NoopT;
//! impl TestingT for NoopT {
//!     fn errorf(&self, _: &str) {}
//!     fn fail_now(&self) {}
//! }
//!
//! let m = Mock::new();
//! m.on("greet", vec![eq("Ada".to_string())])
//!     .returning(|| vec![Box::new("Hello, Ada".to_string())])
//!     .finish();
//!
//! let out = m.method_called("greet", suitecase::mock_args!("Ada".to_string()));
//! assert_eq!(out.string(0), "Hello, Ada");
//! assert!(m.assert_expectations(&NoopT));
//! ```

mod arguments;
mod matchers;

pub use arguments::{Arguments, TestingT};
pub use matchers::{anything, anything_of_type, eq, matched_by, Matcher};

use std::any::Any;
use std::sync::Mutex;
use std::time::Duration;

use arguments::Arguments as Args;

type RunHook = Box<dyn Fn(&Args) + Send + 'static>;

/// Call recorder and expectation registry. Thread-safe (`Mutex`); safe to share across threads.
///
/// # Examples
///
/// See the [module documentation](self).
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

/// First step after [`Mock::on`]: call [`CallBuilder::returning`] to supply return values.
pub struct CallBuilder<'a> {
    mock: &'a Mock,
    method: &'static str,
    matchers: Vec<Box<dyn Matcher>>,
}

/// Builder after [`CallBuilder::returning`]: add [`PostReturn::run`], [`PostReturn::after`], then
/// commit with [`PostReturn::finish`], [`PostReturn::once`], [`PostReturn::times`], [`PostReturn::maybe`], …
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

/// Handle returned after committing an expectation; call [`ActiveExpectation::unset`] to remove it.
pub struct ActiveExpectation<'a> {
    mock: &'a Mock,
    id: usize,
}

impl<'a> CallBuilder<'a> {
    /// Each invocation of the returned closure produces a fresh vector of return values (boxed as
    /// [`Any`](std::any::Any)).
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
    /// Runs before the return closure; receives the same [`Arguments`] as the call.
    pub fn run<R: Fn(&Args) + Send + 'static>(mut self, r: R) -> Self {
        self.run = Some(Box::new(r));
        self
    }

    /// Sleeps for `d` on the calling thread before returning (after the run hook, if any).
    pub fn after(mut self, d: Duration) -> Self {
        self.delay = Some(d);
        self
    }

    /// This expectation does not need to be invoked for [`Mock::assert_expectations`] to pass.
    pub fn maybe(mut self) -> ActiveExpectation<'a> {
        self.maybe = true;
        self.remaining = None;
        self.assert_exact = None;
        self.commit()
    }

    /// Exactly one matching call allowed; assertion requires exactly one invocation.
    pub fn once(mut self) -> ActiveExpectation<'a> {
        self.remaining = Some(1);
        self.assert_exact = Some(1);
        self.commit()
    }

    /// Exactly two matching calls.
    pub fn twice(mut self) -> ActiveExpectation<'a> {
        self.remaining = Some(2);
        self.assert_exact = Some(2);
        self.commit()
    }

    /// Exactly `n` matching calls.
    pub fn times(mut self, n: u32) -> ActiveExpectation<'a> {
        self.remaining = Some(n);
        self.assert_exact = Some(n);
        self.commit()
    }

    /// Unbounded repeat uses of this expectation; assertion still requires at least one call.
    pub fn unlimited(mut self) -> ActiveExpectation<'a> {
        self.remaining = None;
        self.assert_exact = None;
        self.commit()
    }

    /// Same as [`PostReturn::unlimited`].
    pub fn finish(self) -> ActiveExpectation<'a> {
        self.unlimited()
    }

    /// The mocked call panics with `msg` instead of returning values.
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
    /// Removes this expectation so it no longer matches or counts toward assertions.
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
    /// Creates an empty mock with no expectations.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(MockInner {
                expectations: Vec::new(),
                calls: Vec::new(),
                next_id: 0,
            }),
        }
    }

    /// Starts an expectation for `method` with one matcher per argument passed to
    /// [`Mock::method_called`].
    ///
    /// # Examples
    ///
    /// ```
    /// use suitecase::mock::{eq, Mock};
    ///
    /// let m = Mock::new();
    /// m.on("add", vec![eq(1i32), eq(2i32)])
    ///     .returning(|| vec![Box::new(3i32)])
    ///     .finish();
    /// let out = m.method_called("add", suitecase::mock_args!(1i32, 2i32));
    /// assert_eq!(out.int(0), 3);
    /// ```
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

    /// Records a call, finds a matching expectation, runs optional hooks, and returns the values
    /// from the expectation’s return closure.
    ///
    /// # Panics
    ///
    /// Panics if no expectation matches, or if the expectation was configured with
    /// [`PostReturn::panic`].
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

    /// Ensures every non-[`PostReturn::maybe`] expectation was satisfied (invocation counts).
    ///
    /// Returns `false` and notifies `t` on the first failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use suitecase::mock::{Mock, TestingT};
    ///
    /// struct T;
    /// impl TestingT for T {
    ///     fn errorf(&self, _: &str) {}
    ///     fn fail_now(&self) {}
    /// }
    ///
    /// let m = Mock::new();
    /// m.on("x", vec![])
    ///     .returning(|| vec![Box::new(())])
    ///     .finish();
    /// m.method_called("x", suitecase::mock_args!());
    /// assert!(m.assert_expectations(&T));
    /// ```
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

    /// Returns `true` if some recorded call matches `method` and all `matchers`.
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

    /// Returns `true` if no recorded call matches `method` and all `matchers`.
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

    /// Counts recorded calls whose method name equals `method` (all argument patterns).
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

/// Runs [`Mock::assert_expectations`] on each mock in order; stops at the first failure.
///
/// # Examples
///
/// ```
/// use suitecase::mock::{Mock, TestingT};
///
/// struct T;
/// impl TestingT for T {
///     fn errorf(&self, _: &str) {}
///     fn fail_now(&self) {}
/// }
///
/// let a = Mock::new();
/// let b = Mock::new();
/// a.on("a", vec![]).returning(|| vec![Box::new(())]).finish();
/// b.on("b", vec![]).returning(|| vec![Box::new(())]).finish();
/// a.method_called("a", suitecase::mock_args!());
/// b.method_called("b", suitecase::mock_args!());
/// assert!(suitecase::mock::assert_expectations_for_objects(&T, &[&a, &b]));
/// ```
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

/// Builds [`Arguments`] from a list of expressions, each boxed as [`Any`](std::any::Any) + [`Send`].
///
/// Also available as [`suitecase::mock_args`] at the crate root (via [`macro_export`]).
///
/// # Examples
///
/// ```
/// use suitecase::mock::Arguments;
///
/// let a = suitecase::mock_args!(1i32, true, "hi".to_string());
/// assert_eq!(a.len(), 3);
/// let empty = suitecase::mock_args!();
/// assert!(empty.is_empty());
/// ```
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
