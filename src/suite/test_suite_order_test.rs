//! Tests for [`test_suite!`]'s ordering guarantee: each generated `#[test]` runs in registration
//! order regardless of the order cargo's harness picks them, and the shared suite sees every
//! case exactly once in that order.

use crate::{Case, HookFns};

#[derive(Default, Debug)]
struct OrderedState {
    log: Vec<&'static str>,
}

static ORDERED_HOOKS: HookFns<OrderedState> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

static ORDERED_CASES: &[Case<OrderedState>] = cases![OrderedState, s =>
    ord_first => {
        assert_eq!(s.log, Vec::<&'static str>::new(), "ord_first must run before any other case");
        s.log.push("ord_first");
    },
    ord_second => {
        assert_eq!(s.log, vec!["ord_first"], "ord_second must run after ord_first and before ord_third");
        s.log.push("ord_second");
    },
    ord_third => {
        assert_eq!(s.log, vec!["ord_first", "ord_second"], "ord_third must run last");
        s.log.push("ord_third");
    },
];

test_suite!(
    OrderedState,
    ORDERED_SUITE,
    ORDERED_CURSOR,
    OrderedState::default(),
    ORDERED_CASES,
    ORDERED_HOOKS,
    [ord_first, ord_second, ord_third]
);

#[derive(Default, Debug)]
struct InlineOrderedState {
    log: Vec<&'static str>,
}

static INLINE_ORDERED_HOOKS: HookFns<InlineOrderedState> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

test_suite!(
    InlineOrderedState,
    INLINE_ORDERED_SUITE,
    INLINE_ORDERED_CURSOR,
    INLINE_ORDERED_CASES,
    InlineOrderedState::default(),
    INLINE_ORDERED_HOOKS,
    s =>
    inline_ord_a => {
        assert_eq!(s.log, Vec::<&'static str>::new());
        s.log.push("inline_ord_a");
    },
    inline_ord_b => {
        assert_eq!(s.log, vec!["inline_ord_a"]);
        s.log.push("inline_ord_b");
    },
);
