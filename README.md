# Suitcase

**The structured test toolkit.** A lightweight sync Rust library for **named cases**, optional **setup** / **teardown** at suite scope and **before_each** / **after_each** around each case. Build case lists with [`cases!`](https://docs.rs/suitecase/latest/suitecase/macro.cases.html); use [`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html) to emit a single `#[test]` that runs all cases sequentially with formatted output.

**Heavy development:** The API is still evolving. Expect **breaking changes** between releases until a stable 1.0; pin an exact version (or git revision) in `Cargo.toml` if you need upgrades to be predictable.

**Install · [Usage](#usage) · [CLI](#cli) · [Case Selection](#case-selection) · [Dependencies](#case-dependencies-depends_on) · [Fail](#fail-and-fail_now) · [Assertions](#assertions-assert) · [Mocking](#mocking-mock) · [Examples](#examples) · [AI-assisted changes](AI_USAGE.md) · [Docs](https://docs.rs/suitecase)** (after publish; `cargo doc --open` locally)

---

- **Sync runner** — [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html) orchestrates hooks and case bodies; integrate async I/O with something like `tokio::runtime::Handle::block_on` in hooks or cases.
- **Sequential execution** — [`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html) emits **one** `#[test]` that runs all cases in slice order. Cases that depend on earlier state (setup → mutate → assert) always execute correctly.
- **Formatted output** — Each case prints `▶ name`, then `✓ name (Xms)` or `✗ name (Xms)`. All cases run even if one fails; the first panic is re-raised after completion.
- **Hooks as optional fns** — [`HookFns`](https://docs.rs/suitecase/latest/suitecase/suite/struct.HookFns.html) holds `Option<fn(&mut S)>` per lifecycle slot; use [`None`] to skip.
- **Case selection** — [`RunConfig::filter`](https://docs.rs/suitecase/latest/suitecase/suite/struct.RunConfig.html#method.filter), [`RunConfig::filters`](https://docs.rs/suitecase/latest/suitecase/suite/struct.RunConfig.html#method.filters), and [`RunConfig::pattern`](https://docs.rs/suitecase/latest/suitecase/suite/struct.RunConfig.html#method.pattern) let you run a single case, a group, or a glob match. Dependencies declared via `depends_on` are auto-included.
- **CLI** — `suitecase test` runs `suitecase test` and renders a formatted summary with pass/fail counts, per-case timing, and failure details.
- **Assertions** — [`suitecase::assert`](https://docs.rs/suitecase/latest/suitecase/assert/index.html) provides [testify/assert](https://pkg.go.dev/github.com/stretchr/testify/assert)-style helpers (`equal`, `contains`, `assert_ok`, …) that **panic** on failure with clear messages ([`#[track_caller]`](https://doc.rust-lang.org/stable/std/panic/struct.Location.html) where applicable).
- **Mocking** — [`suitecase::mock`](https://docs.rs/suitecase/latest/suitecase/mock/index.html) provides [testify/mock](https://pkg.go.dev/github.com/stretchr/testify/mock)-style **expectations** and **call recording** ([`Mock`](https://docs.rs/suitecase/latest/suitecase/mock/struct.Mock.html), [`mock_args!`](https://docs.rs/suitecase/latest/suitecase/macro.mock_args.html)). Use [`suitecase::mock_args`](https://docs.rs/suitecase/latest/suitecase/macro.mock_args.html) at the crate root (same macro).

---

## Install

Add **suitecase** to your `Cargo.toml` (no required dependencies beyond `std`):

```toml
[dependencies]
suitecase = "0.1"
```

Install the CLI:

```sh
cargo install suitecase
```

---

## CLI

Run tests with formatted suitecase output:

```sh
suitecase test --lib
suitecase test --example quickstart
suitecase test --example sqlx_sqlite
suitecase test --lib -- shared_state    # filter by test name
```

Output:

```
Suitecase Summary
────────────────────────────────────────────────
  ✓ test_inc (0ms)
  ✓ test_inc_verify (0ms)
  ✓ test_double (0ms)
  ...
────────────────────────────────────────────────
  PASSED  12 passed, 0 failed  (total: 0ms)
```

On failure, a **FAILURES** section shows the panicked case name, duration, file location, and assertion message.

---

## Usage

### Quickstart

Put this in `tests/suite.rs` or `#[cfg(test)] mod tests { ... }`. The [example](examples/quickstart.rs) is `suitecase test --example quickstart` — **one** test line that runs every case in order on **one** suite.

```rust
use suitecase::{cases, run, Case, HookFns, RunConfig};

#[derive(Default)]
struct Counter {
    n: i32,
}

impl Counter {
    fn test_inc(&mut self) {
        self.n += 1;
    }
    fn test_inc_verify(&mut self) {
        assert_eq!(self.n, 1);
    }
}

static MY_CASES: &[Case<Counter>] = cases![Counter, s =>
    test_inc => { s.test_inc(); },
    test_inc_verify => { s.test_inc_verify(); },
];

static MY_HOOKS: HookFns<Counter> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

#[test]
fn quickstart() {
    let mut suite = Counter::default();
    run(&mut suite, MY_CASES, RunConfig::all(), &MY_HOOKS);
}
```

Run: `suitecase test`.

### Hooks

Pass [`HookFns`](https://docs.rs/suitecase/latest/suitecase/suite/struct.HookFns.html) as the last argument to [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html). Each field is `Option<fn(&mut S)>` — [`Some(...)]` or [`None`] / [`HookFns::default()`].

### Case selection

Run a single case, a group, or a glob pattern by passing a [`RunConfig`](https://docs.rs/suitecase/latest/suitecase/suite/struct.RunConfig.html) to [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html). Hooks (`setup_suite`, `teardown_suite`, `before_each`, `after_each`) still execute for selected cases.

```rust
use suitecase::{cases, run, Case, HookFns, RunConfig};

#[derive(Default)]
struct Counter { n: i32 }

static CASES: &[Case<Counter>] = cases![Counter, s =>
    test_a => { s.n = 1; },
    test_b => { s.n = 2; },
    test_c => { s.n = 3; },
];

// Single case
run(&mut Counter::default(), CASES, RunConfig::filter("test_b"), &HookFns::default());

// Multiple cases
run(&mut Counter::default(), CASES, RunConfig::filters(["test_a".to_string(), "test_c".to_string()]), &HookFns::default());

// Glob pattern (* matches any characters)
run(&mut Counter::default(), CASES, RunConfig::pattern("test_*"), &HookFns::default());
```

### Case dependencies (`depends_on`)

Declare that a case requires other cases to run first. When you filter to a case with dependencies, those dependencies are automatically included and run in topological order.

```rust
use suitecase::{cases, run, Case, HookFns, RunConfig};

#[derive(Default)]
struct Db { connected: bool, migrated: bool }

static CASES: &[Case<Db>] = cases![Db, s =>
    connect => { s.connected = true; },
    migrate(depends_on = [connect]) => { s.migrated = true; },
    query(depends_on = [migrate]) => { assert!(s.connected && s.migrated); },
];

// Running only "query" auto-includes connect → migrate → query
let mut db = Db::default();
run(&mut db, CASES, RunConfig::filter("query"), &HookFns::default());
```

Circular dependencies and missing dependencies **panic** at runtime.

### Fail and fail_now

Use [`suitecase::fail`](https://docs.rs/suitecase/latest/suitecase/fn.fail.html) and [`suitecase::fail_now`](https://docs.rs/suitecase/latest/suitecase/fn.fail_now.html) inside case bodies:

- **`fail(msg)`** — marks the current case as failed, **continues** running remaining cases, exits cleanly (no re-panic).
- **`fail_now(msg)`** — marks the current case as failed, **aborts** all remaining cases, runs `teardown_suite`, then panics.

If a case fails via either function, cases that depend on it (via `depends_on`) are **skipped** (shown as `⊘`).

```rust
use suitecase::{cases, run, Case, HookFns, RunConfig, fail};

#[derive(Default)]
struct App { ready: bool }

static CASES: &[Case<App>] = cases![App, s =>
    setup => { s.ready = true; },
    check => { if !s.ready { fail("not ready"); } },
];
```

### Sequential test suite (`test_suite!`)

[`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html) emits a **single** `#[test]` that runs all cases sequentially in slice order. Each case prints `▶ name` before execution and `✓ name (Xms)` or `✗ name (Xms)` after. All cases run even if one fails — the first panic is re-raised after all cases complete.

```rust
use suitecase::{test_suite, HookFns};

#[derive(Default)]
struct Counter { n: i32 }

test_suite!(
    Counter,
    MY_SHARED_SUITE,
    quickstart_test_run,
    Counter::default(),
    HookFns::default(),
    s =>
    test_inc => { s.n += 1; },
    test_inc_verify => { assert_eq!(s.n, 1); },
);
```

Run: `suitecase test` or `suitecase test`.

### Assertions (`assert`)

Import helpers from [`suitecase::assert`](https://docs.rs/suitecase/latest/suitecase/assert/index.html) inside `#[test]` functions, suite case bodies, or anywhere you want a **named** check that fails by **panicking** with a readable message.

```rust
use suitecase::assert::{assert_ok, contains_str, equal};

let n = assert_ok(Ok::<_, &str>(42));
equal(&n, &42);
contains_str("hello world", "world");
```

See the module docs for the full flat list (collections, errors, floats, options/results, ordering, panics, pointers, time, …).

### Mocking (`mock`)

[`suitecase::mock`](https://docs.rs/suitecase/latest/suitecase/mock/index.html) is for **test doubles**: register expectations on a [`Mock`](https://docs.rs/suitecase/latest/suitecase/mock/struct.Mock.html), forward calls from your stub with [`method_called`](https://docs.rs/suitecase/latest/suitecase/mock/struct.Mock.html#method.method_called) and [`mock_args!`](https://docs.rs/suitecase/latest/suitecase/macro.mock_args.html), then verify with [`assert_expectations`](https://docs.rs/suitecase/latest/suitecase/mock/struct.Mock.html#method.assert_expectations) and a small [`TestingT`](https://docs.rs/suitecase/latest/suitecase/mock/trait.TestingT.html) implementation.

```rust
use suitecase::mock::{eq, Mock, TestingT};

struct NoopT;
impl TestingT for NoopT {
    fn errorf(&self, _: &str) {}
    fn fail_now(&self) {}
}

let m = Mock::new();
m.on("greet", vec![eq("Ada".to_string())])
    .returning(|| vec![Box::new("Hello, Ada".to_string())])
    .finish();

let out = m.method_called("greet", suitecase::mock_args!("Ada".to_string()));
assert_eq!(out.string(0), "Hello, Ada");
assert!(m.assert_expectations(&NoopT));
```

Unexpected calls **panic** from [`method_called`](https://docs.rs/suitecase/latest/suitecase/mock/struct.Mock.html#method.method_called) (see rustdoc). Matchers include [`eq`](https://docs.rs/suitecase/latest/suitecase/mock/fn.eq.html), [`anything`](https://docs.rs/suitecase/latest/suitecase/mock/fn.anything.html), [`anything_of_type`](https://docs.rs/suitecase/latest/suitecase/mock/fn.anything_of_type.html), [`matched_by`](https://docs.rs/suitecase/latest/suitecase/mock/fn.matched_by.html).

---

## Examples

| Example | Run |
|--------|-----|
| [`examples/quickstart.rs`](examples/quickstart.rs) | `suitecase test --example quickstart` · `suitecase test --example quickstart` |
| [`examples/mock.rs`](examples/mock.rs) | `cargo run --example mock` · `suitecase test --example mock` (mocked HTTP-style JSON) |
| [`examples/sqlx_sqlite.rs`](examples/sqlx_sqlite.rs) | `suitecase test --example sqlx_sqlite` · `suitecase test --example sqlx_sqlite` |
| Every example at once | `suitecase test --examples` |
| Integration tests ([`tests/suite.rs`](tests/suite.rs)) | `suitecase test` |

The **sqlx** example uses **sqlx** + **tokio**, embedded migrations in `setup_suite`, and [`cases!`](https://docs.rs/suitecase/latest/suitecase/macro.cases.html) blocks that call helper functions, plus [`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html).

```toml
[dependencies]
suitecase = "0.1"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

---

## More documentation

- **[`AI_USAGE.md`](AI_USAGE.md)** — using LLMs to draft changes is allowed; you must understand the patch, justify it in review, and avoid unnecessary complexity (see file).
- **`cargo doc --open`** — [`cases!`](https://docs.rs/suitecase/latest/suitecase/macro.cases.html) and [`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html) include declaration / description / examples.
- Crate root and [`suite`](https://docs.rs/suitecase/latest/suitecase/suite/index.html) describe how [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html) selects cases and orders hooks.
- [`assert`](https://docs.rs/suitecase/latest/suitecase/assert/index.html) — panicking assertion helpers (submodules per domain; flat re-exports at the module root).
- [`mock`](https://docs.rs/suitecase/latest/suitecase/mock/index.html) — `Mock`, matchers, [`Arguments`](https://docs.rs/suitecase/latest/suitecase/mock/struct.Arguments.html), [`mock_args!`](https://docs.rs/suitecase/latest/suitecase/macro.mock_args.html).
