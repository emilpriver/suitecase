# Suitcase

The library package name is **`suitcase`** — use `suitcase = "0.1"` in `Cargo.toml` (or `path` / `git` as below).

Structured test suites for Rust: **setup** / **teardown** at suite scope and **before_each** / **after_each** around each case, with optional filtering so a single case still runs the right hooks.

**Install · [Usage](#usage) · [Examples](#examples) · [Docs](https://docs.rs/suitcase)** (after publish; `cargo doc --open` locally)

---

- **Sync runner** — [`run`](https://docs.rs/suitcase/latest/suitcase/suite/fn.run.html) orchestrates hooks and case bodies; integrate async I/O with something like `tokio::runtime::Handle::block_on` in hooks or cases.
- **Hooks as optional fns** — [`HookFns`](https://docs.rs/suitcase/latest/suitcase/suite/struct.HookFns.html) holds `Option<fn(&mut S)>` per lifecycle slot; use [`None`] to skip.
- **Named cases** — build `&'static [Case<S>]` with [`suite_methods!`](https://docs.rs/suitcase/latest/suitcase/macro.suite_methods.html), [`cases!`](https://docs.rs/suitcase/latest/suitcase/macro.cases.html), or [`cases_fn!`](https://docs.rs/suitcase/latest/suitcase/macro.cases_fn.html).
- **One line per case in `cargo test`** — [`cargo_case_tests!`](https://docs.rs/suitcase/latest/suitcase/macro.cargo_case_tests.html) / [`cargo_case_tests_with_hooks!`](https://docs.rs/suitcase/latest/suitcase/macro.cargo_case_tests_with_hooks.html) emit a `#[test]` per case name (each run uses [`RunConfig::filter`](https://docs.rs/suitcase/latest/suitcase/suite/struct.RunConfig.html#method.filter)).

---

## Install

Add **suitcase** to your `Cargo.toml` (no required dependencies beyond `std`):

```toml
[dependencies]
suitcase = "0.1"
```

---

## Usage

### Quickstart

```rust
use suitcase::{run, suite_methods, Case, HookFns, RunConfig};

#[derive(Default)]
struct Counter {
    n: i32,
}

impl Counter {
    fn test_inc(&mut self) {
        self.n += 1;
    }
}

static CASES: &[Case<Counter>] = suite_methods![Counter, s => test_inc];

fn main() {
    let mut suite = Counter::default();
    run(&mut suite, CASES, RunConfig::all(), &HookFns::default());
    assert_eq!(suite.n, 1);
}
```

### Hooks

Pass [`HookFns`](https://docs.rs/suitcase/latest/suitcase/suite/struct.HookFns.html) as the last argument to [`run`](https://docs.rs/suitcase/latest/suitcase/suite/fn.run.html). Each field is `Option<fn(&mut S)>` — wrap your function in [`Some(...)`] or use [`None`] / [`HookFns::default()`] to skip.

```rust
use suitcase::{run, suite_methods, Case, HookFns, RunConfig};

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

static CASES: &[Case<State>] = suite_methods![State, s => test_ok];

static HOOKS: HookFns<State> = HookFns {
    setup_suite: Some(setup),
    teardown_suite: None,
    before_each: None,
    after_each: None,
};

fn main() {
    let mut suite = State::default();
    run(&mut suite, CASES, RunConfig::all(), &HOOKS);
    assert_eq!(suite.log, vec!["setup", "case"]);
}
```

### Run a single case (filter)

```rust
use suitcase::{run, suite_methods, Case, HookFns, RunConfig};

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

static CASES: &[Case<Counter>] = suite_methods![Counter, s => test_a, test_b];

fn main() {
    let mut suite = Counter::default();
    run(
        &mut suite,
        CASES,
        RunConfig::filter("test_b"),
        &HookFns::default(),
    );
    assert_eq!(suite.n, 2);
}
```

### Show each case in `cargo test`

Rust lists one line per `#[test]`. To list **each case** separately, use [`cargo_case_tests!`](https://docs.rs/suitcase/latest/suitcase/macro.cargo_case_tests.html) (default hooks) or [`cargo_case_tests_with_hooks!`](https://docs.rs/suitcase/latest/suitcase/macro.cargo_case_tests_with_hooks.html) at the **root** of an integration test file (`tests/*.rs`):

```rust
use suitcase::{cargo_case_tests_with_hooks, suite_methods, Case, HookFns};

// static MY_CASES: &[Case<MySuite>] = suite_methods![MySuite, s => test_a, test_b];
// static MY_HOOKS: HookFns<MySuite> = HookFns { /* ... */ };

cargo_case_tests_with_hooks!(MySuite::default(), MY_CASES, MY_HOOKS, [test_a, test_b]);
```

Each case body should be correct when it is the **only** case selected (fresh suite state unless you seed in the body).

---

## Examples

| Example | Run |
|--------|-----|
| [`examples/sqlx_sqlite.rs`](examples/sqlx_sqlite.rs) | `cargo run --example sqlx_sqlite` |

That example uses **sqlx** + **tokio**, applies embedded migrations in `setup_suite`, and runs several free-function cases with [`cases_fn!`](https://docs.rs/suitcase/latest/suitcase/macro.cases_fn.html) and [`HookFns`](https://docs.rs/suitcase/latest/suitcase/suite/struct.HookFns.html). Enable the same stack in your crate if you copy the pattern:

```toml
[dependencies]
suitcase = "0.1"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Integration tests in this repo under [`tests/`](tests/) mirror the same APIs (`tests/suite.rs`, `tests/sqlx_sqlite.rs`).

---

## More documentation

- **`cargo doc --open`** — full API, execution model, and macro **Declaration / Description / Example** sections.
- Crate root and [`suite`](https://docs.rs/suitcase/latest/suitcase/suite/index.html) module describe how [`run`](https://docs.rs/suitcase/latest/suitcase/suite/fn.run.html) selects cases and orders hooks.
