#[derive(Clone, Copy, Debug)]
pub struct HookFns<S> {
    pub setup_suite: Option<fn(&mut S)>,
    pub teardown_suite: Option<fn(&mut S)>,
    pub before_each: Option<fn(&mut S)>,
    pub after_each: Option<fn(&mut S)>,
}

impl<S> Default for HookFns<S> {
    fn default() -> Self {
        Self {
            setup_suite: None,
            teardown_suite: None,
            before_each: None,
            after_each: None,
        }
    }
}

impl<S> HookFns<S> {
    pub fn noop() -> Self {
        Self::default()
    }
}

pub struct Case<S> {
    pub name: &'static str,
    pub run: fn(&mut S),
}

impl<S> Case<S> {
    pub const fn new(name: &'static str, run: fn(&mut S)) -> Self {
        Self { name, run }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunConfig {
    pub filter: Option<String>,
}

impl RunConfig {
    pub fn all() -> Self {
        Self::default()
    }

    pub fn filter(name: impl Into<String>) -> Self {
        Self {
            filter: Some(name.into()),
        }
    }

    pub fn from_env() -> Self {
        Self {
            filter: std::env::var("SUITCASE_FILTER")
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }
}

pub fn run<S>(suite: &mut S, cases: &[Case<S>], config: RunConfig, hooks: &HookFns<S>) {
    run_hooks(
        suite,
        cases,
        config,
        |s| {
            if let Some(f) = hooks.setup_suite {
                f(s);
            }
        },
        |s| {
            if let Some(f) = hooks.teardown_suite {
                f(s);
            }
        },
        |s| {
            if let Some(f) = hooks.before_each {
                f(s);
            }
        },
        |s| {
            if let Some(f) = hooks.after_each {
                f(s);
            }
        },
    );
}

fn run_hooks<S, FS, FT, FB, FA>(
    suite: &mut S,
    cases: &[Case<S>],
    config: RunConfig,
    mut setup_suite: FS,
    mut teardown_suite: FT,
    mut before_each: FB,
    mut after_each: FA,
) where
    FS: FnMut(&mut S),
    FT: FnMut(&mut S),
    FB: FnMut(&mut S),
    FA: FnMut(&mut S),
{
    let selected: Vec<&Case<S>> = cases
        .iter()
        .filter(|c| match &config.filter {
            None => true,
            Some(f) => c.name == f,
        })
        .collect();

    if selected.is_empty() {
        assert!(
            config.filter.is_none(),
            "suitcase: filter {:?} matched no cases",
            config.filter
        );
        return;
    }

    setup_suite(suite);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for case in selected {
            before_each(suite);
            let case_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                (case.run)(suite);
            }));
            after_each(suite);
            if let Err(payload) = case_result {
                std::panic::resume_unwind(payload);
            }
        }
    }));
    teardown_suite(suite);
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

#[macro_export]
macro_rules! suite_methods {
    ($ty:ty, $s:ident => $($name:ident),* $(,)?) => {
        &[$( $crate::suite::Case::<$ty>::new(stringify!($name), |$s: &mut $ty| { $s.$name(); })),*]
    };
}

#[macro_export]
macro_rules! cases {
    ($ty:ty, $s:ident => $($name:ident => $body:block),* $(,)?) => {
        &[$( $crate::suite::Case::<$ty>::new(stringify!($name), |$s: &mut $ty| $body)),*]
    };
}

#[macro_export]
macro_rules! cases_fn {
    ($ty:ty => $($name:ident => $fn:path),* $(,)?) => {
        &[$( $crate::suite::Case::<$ty>::new(stringify!($name), $fn as fn(&mut $ty))),*]
    };
}

#[macro_export]
macro_rules! cargo_case_tests {
    ($suite:expr, $cases:expr, [$($name:ident),* $(,)?] $(,)?) => {
        $(
            #[test]
            fn $name() {
                let mut suite = $suite;
                $crate::suite::run(
                    &mut suite,
                    $cases,
                    $crate::suite::RunConfig::filter(stringify!($name)),
                    &$crate::suite::HookFns::default(),
                );
            }
        )*
    };
}

#[macro_export]
macro_rules! cargo_case_tests_with_hooks {
    ($suite:expr, $cases:expr, $hooks:expr, [$($name:ident),* $(,)?] $(,)?) => {
        $(
            #[test]
            fn $name() {
                let mut suite = $suite;
                $crate::suite::run(
                    &mut suite,
                    $cases,
                    $crate::suite::RunConfig::filter(stringify!($name)),
                    &$hooks,
                );
            }
        )*
    };
}
