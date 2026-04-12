# Suitcase

**The structured test toolkit.** A lightweight sync Rust library for **named cases**, optional **setup** / **teardown** at suite scope and **before_each** / **after_each** around each case, plus macros so every case can show up as its own line in **cargo test**—without a custom harness or DSL. Filter to a single case when you want isolation; hooks still run in the right order.

**Heavy development:** The API is still evolving. Expect **breaking changes** between releases until a stable 1.0; pin an exact version (or git revision) in `Cargo.toml` if you need upgrades to be predictable.

**Install · [Usage](#usage) · [Examples](#examples) · [Docs](https://docs.rs/suitecase)** (after publish; `cargo doc --open` locally)

---

- **Sync runner** — [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html) orchestrates hooks and case bodies; integrate async I/O with something like `tokio::runtime::Handle::block_on` in hooks or cases.
- **Hooks as optional fns** — [`HookFns`](https://docs.rs/suitecase/latest/suitecase/suite/struct.HookFns.html) holds `Option<fn(&mut S)>` per lifecycle slot; use [`None`] to skip.
- **Named cases** — build `&'static [Case<S>]` with [`suite_methods!`](https://docs.rs/suitecase/latest/suitecase/macro.suite_methods.html), [`cases!`](https://docs.rs/suitecase/latest/suitecase/macro.cases.html), or [`cases_fn!`](https://docs.rs/suitecase/latest/suitecase/macro.cases_fn.html).
- **One line per case in `cargo test`** — [`cargo_case_tests!`](https://docs.rs/suitecase/latest/suitecase/macro.cargo_case_tests.html) / [`cargo_case_tests_with_hooks!`](https://docs.rs/suitecase/latest/suitecase/macro.cargo_case_tests_with_hooks.html) emit a `#[test]` per case name (each run uses [`RunConfig::filter`](https://docs.rs/suitecase/latest/suitecase/suite/struct.RunConfig.html#method.filter)).

---

## Install

Add **suitecase** to your `Cargo.toml` (no required dependencies beyond `std`):

```toml
[dependencies]
suitecase = "0.1"
```

---

## Usage

### Quickstart

Put this in `tests/suite.rs` (integration test) or inside `#[cfg(test)] mod tests { ... }` in your crate. The same flow is an [example target](examples/quickstart.rs) — `cargo test --example quickstart`.

```rust
use suitecase::{cargo_case_tests_with_hooks, suite_methods, Case, HookFns};

#[derive(Default)]
struct Counter {
    n: i32,
}

impl Counter {
    fn test_inc(&mut self) {
        self.n += 1;
    }
}

static MY_CASES: &[Case<Counter>] = suite_methods![Counter, s => test_inc];

static MY_HOOKS: HookFns<Counter> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

cargo_case_tests_with_hooks!(Counter::default(), MY_CASES, MY_HOOKS, [test_inc]);
```

Run: `cargo test` — one `#[test]` per name in the list (`[test_inc]` here).

### Hooks

Pass [`HookFns`](https://docs.rs/suitecase/latest/suitecase/suite/struct.HookFns.html) as the last argument to [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html). Each field is `Option<fn(&mut S)>` — wrap your function in [`Some(...)`] or use [`None`] / [`HookFns::default()`] to skip.

Runnable: [`examples/hooks.rs`](examples/hooks.rs) — `cargo test --example hooks`.

```rust
use suitecase::{cargo_case_tests_with_hooks, suite_methods, Case, HookFns};

#[derive(Default)]
struct State {
    log: Vec<&'static str>,
}

impl State {
    fn test_ok(&mut self) {
        self.log.push("case");
    }
}

fn setup(s: &mut State) {
    s.log.push("setup");
}

static MY_CASES: &[Case<State>] = suite_methods![State, s => test_ok];

static MY_HOOKS: HookFns<State> = HookFns {
    setup_suite: Some(setup),
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

cargo_case_tests_with_hooks!(State::default(), MY_CASES, MY_HOOKS, [test_ok]);
```

Run: `cargo test`

### Run a single case (filter)

Each generated test calls [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html) with [`RunConfig::filter`](https://docs.rs/suitecase/latest/suitecase/suite/struct.RunConfig.html#method.filter) for that case name — same effect as listing several cases in [`cargo_case_tests_with_hooks!`](https://docs.rs/suitecase/latest/suitecase/macro.cargo_case_tests_with_hooks.html).

Runnable: [`examples/filter.rs`](examples/filter.rs) — `cargo test --example filter`.

```rust
use suitecase::{cargo_case_tests_with_hooks, suite_methods, Case, HookFns};

#[derive(Default)]
struct Counter {
    n: i32,
}

impl Counter {
    fn test_a(&mut self) {
        self.n = 1;
    }
    fn test_b(&mut self) {
        self.n = 2;
    }
}

static MY_CASES: &[Case<Counter>] = suite_methods![Counter, s => test_a, test_b];

static MY_HOOKS: HookFns<Counter> = HookFns {
    setup_suite: None,
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

cargo_case_tests_with_hooks!(Counter::default(), MY_CASES, MY_HOOKS, [test_a, test_b]);
```

Run: `cargo test`

### Show each case in `cargo test`

Rust lists one line per `#[test]`. Use [`cargo_case_tests!`](https://docs.rs/suitecase/latest/suitecase/macro.cargo_case_tests.html) (default hooks) or [`cargo_case_tests_with_hooks!`](https://docs.rs/suitecase/latest/suitecase/macro.cargo_case_tests_with_hooks.html) at the **root** of an integration test file (`tests/*.rs`) or under [`examples/`](examples/) — **all cookbook examples** in this repo use `cargo_case_tests_with_hooks!(suite, MY_CASES, MY_HOOKS, […])`.

Runnable: [`examples/per_case_tests.rs`](examples/per_case_tests.rs) — `cargo test --example per_case_tests`.

```rust
use suitecase::{cargo_case_tests_with_hooks, suite_methods, Case, HookFns};

// static MY_CASES: &[Case<MySuite>] = suite_methods![MySuite, s => test_a, test_b];
// static MY_HOOKS: HookFns<MySuite> = HookFns { /* ... */ };

cargo_case_tests_with_hooks!(MySuite::default(), MY_CASES, MY_HOOKS, [test_a, test_b]);
```

In `tests/*.rs`, run `cargo test` — you should see one test per listed case name.

Each case body should be correct when it is the **only** case selected (fresh suite state unless you seed in the body).

---

## Examples

| Example | Run |
|--------|-----|
| [`examples/quickstart.rs`](examples/quickstart.rs) | `cargo test --example quickstart` |
| [`examples/hooks.rs`](examples/hooks.rs) | `cargo test --example hooks` |
| [`examples/filter.rs`](examples/filter.rs) | `cargo test --example filter` |
| [`examples/per_case_tests.rs`](examples/per_case_tests.rs) | `cargo test --example per_case_tests` |
| [`examples/sqlx_sqlite.rs`](examples/sqlx_sqlite.rs) | `cargo test --example sqlx_sqlite` |
| Every example at once | `cargo test --examples` |
| Integration tests in this repo ([`tests/`](tests/)) | `cargo test` |

The **sqlx** example uses **sqlx** + **tokio**, applies embedded migrations in `setup_suite`, and runs several free-function cases with [`cases_fn!`](https://docs.rs/suitecase/latest/suitecase/macro.cases_fn.html) and [`HookFns`](https://docs.rs/suitecase/latest/suitecase/suite/struct.HookFns.html). Enable the same stack in your crate if you copy the pattern:

```toml
[dependencies]
suitecase = "0.1"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

See [`tests/suite.rs`](tests/suite.rs) and [`tests/sqlx_sqlite.rs`](tests/sqlx_sqlite.rs) for full implementations of these patterns.

---

## More documentation

- **`cargo doc --open`** — full API, execution model, and macro **Declaration / Description / Example** sections.
- Crate root and [`suite`](https://docs.rs/suitecase/latest/suitecase/suite/index.html) module describe how [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html) selects cases and orders hooks.
