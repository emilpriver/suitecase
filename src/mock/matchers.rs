//! Argument [matchers](Matcher) used with [`Mock::on`](crate::Mock::on) and
//! [`Arguments::assert_matches`](crate::Arguments::assert_matches).
//!
//! Prefer the helpers [`anything`], [`anything_of_type`], [`eq`], and [`matched_by`].

use std::any::{Any, TypeId};

/// Predicate used to compare one boxed argument against an expectation.
///
pub trait Matcher: Send {
    /// Returns `true` when this matcher accepts `arg`.
    fn matches(&self, arg: &dyn Any) -> bool;
}

/// Matches any value.
pub struct Anything;

impl Matcher for Anything {
    fn matches(&self, _arg: &dyn Any) -> bool {
        true
    }
}

/// Wildcard matcher: always matches.
///
/// # Examples
///
/// ```
/// use suitecase::mock::{anything, Mock};
///
/// let m = Mock::new();
/// m.on("x", vec![anything()])
///     .returning(|| vec![Box::new(0u8)])
///     .finish();
/// m.method_called("x", suitecase::mock_args!("anything goes"));
/// ```
pub fn anything() -> Box<dyn Matcher> {
    Box::new(Anything)
}

/// Matches when the dynamic type equals `T` ([`TypeId`](std::any::TypeId)).
pub struct OfType {
    id: TypeId,
}

/// Matches any value whose concrete type is `T`.
///
/// # Examples
///
/// ```
/// use suitecase::mock::{anything_of_type, Mock};
///
/// let m = Mock::new();
/// m.on("x", vec![anything_of_type::<i32>()])
///     .returning(|| vec![Box::new(())])
///     .finish();
/// m.method_called("x", suitecase::mock_args!(42i32));
/// ```
pub fn anything_of_type<T: 'static>() -> Box<dyn Matcher> {
    Box::new(OfType {
        id: TypeId::of::<T>(),
    })
}

impl Matcher for OfType {
    fn matches(&self, arg: &dyn Any) -> bool {
        arg.type_id() == self.id
    }
}

/// Equality matcher for a concrete `T` ([`PartialEq`]).
pub struct EqMatcher<T: PartialEq + Send + 'static>(pub T);

impl<T: PartialEq + Send + 'static> Matcher for EqMatcher<T> {
    fn matches(&self, arg: &dyn Any) -> bool {
        arg.downcast_ref::<T>()
            .map(|v| *v == self.0)
            .unwrap_or(false)
    }
}

/// Matches when the argument downcasts to `T` and compares equal to `v`.
///
/// # Examples
///
/// ```
/// use suitecase::mock::{eq, Mock};
///
/// let m = Mock::new();
/// m.on("id", vec![eq(99u32)])
///     .returning(|| vec![Box::new("ok".to_string())])
///     .finish();
/// let out = m.method_called("id", suitecase::mock_args!(99u32));
/// assert_eq!(out.string(0), "ok");
/// ```
pub fn eq<T: PartialEq + Send + 'static>(v: T) -> Box<dyn Matcher> {
    Box::new(EqMatcher(v))
}

/// Matcher wrapping a custom predicate over [`Any`].
pub struct MatchedBy<F>
where
    F: Fn(&dyn Any) -> bool + Send,
{
    pub f: F,
}

impl<F> Matcher for MatchedBy<F>
where
    F: Fn(&dyn Any) -> bool + Send,
{
    fn matches(&self, arg: &dyn Any) -> bool {
        (self.f)(arg)
    }
}

/// Custom match: `f` receives the erased argument and returns whether it matches.
///
/// # Examples
///
/// ```
/// use suitecase::mock::{matched_by, Mock};
///
/// let m = Mock::new();
/// m.on("x", vec![matched_by(|a| a.downcast_ref::<i32>().map(|v| *v > 0).unwrap_or(false))])
///     .returning(|| vec![Box::new(1i32)])
///     .finish();
/// let out = m.method_called("x", suitecase::mock_args!(7i32));
/// assert_eq!(out.int(0), 1i64);
/// ```
pub fn matched_by<F>(f: F) -> Box<dyn Matcher>
where
    F: Fn(&dyn Any) -> bool + Send + 'static,
{
    Box::new(MatchedBy { f })
}
