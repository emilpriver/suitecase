use std::any::{Any, TypeId};

pub trait Matcher: Send {
    fn matches(&self, arg: &dyn Any) -> bool;
}

pub struct Anything;

impl Matcher for Anything {
    fn matches(&self, _arg: &dyn Any) -> bool {
        true
    }
}

pub fn anything() -> Box<dyn Matcher> {
    Box::new(Anything)
}

pub struct OfType {
    id: TypeId,
}

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

pub struct EqMatcher<T: PartialEq + Send + 'static>(pub T);

impl<T: PartialEq + Send + 'static> Matcher for EqMatcher<T> {
    fn matches(&self, arg: &dyn Any) -> bool {
        arg.downcast_ref::<T>()
            .map(|v| *v == self.0)
            .unwrap_or(false)
    }
}

pub fn eq<T: PartialEq + Send + 'static>(v: T) -> Box<dyn Matcher> {
    Box::new(EqMatcher(v))
}

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

pub fn matched_by<F>(f: F) -> Box<dyn Matcher>
where
    F: Fn(&dyn Any) -> bool + Send + 'static,
{
    Box::new(MatchedBy { f })
}
