# Suitcase

**The structured test toolkit.** A lightweight sync Rust library for **named cases**, optional **setup** / **teardown** at suite scope and **before_each** / **after_each** around each case. Build case lists with [`cases!`](https://docs.rs/suitecase/latest/suitecase/macro.cases.html); use [`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html) so each case appears as its own line in **`cargo test`**, with all those tests sharing one suite behind a [`Mutex`](https://doc.rust-lang.org/stable/std/sync/struct.Mutex.html).

**Heavy development:** The API is still evolving. Expect **breaking changes** between releases until a stable 1.0; pin an exact version (or git revision) in `Cargo.toml` if you need upgrades to be predictable.

**Install · [Usage](#usage) · [Examples](#examples) · [Docs](https://docs.rs/suitecase)** (after publish; `cargo doc --open` locally)

---

- **Sync runner** — [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html) orchestrates hooks and case bodies; integrate async I/O with something like `tokio::runtime::Handle::block_on` in hooks or cases.
- **Hooks as optional fns** — [`HookFns`](https://docs.rs/suitecase/latest/suitecase/suite/struct.HookFns.html) holds `Option<fn(&mut S)>` per lifecycle slot; use [`None`] to skip.
- **Macros** — [`cases!`](https://docs.rs/suitecase/latest/suitecase/macro.cases.html) builds `&'static [Case<S>]`. [`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html) emits one `#[test]` per case name; each run uses [`RunConfig::filter`](https://docs.rs/suitecase/latest/suitecase/suite/struct.RunConfig.html#method.filter) and the **same** shared suite (see macro docs for `Send` and ordering).

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

Put this in `tests/suite.rs` or `#[cfg(test)] mod tests { ... }`. The [example](examples/quickstart.rs) is `cargo test --example quickstart` — **one** test line that runs every case in order on **one** suite.

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

Run: `cargo test`.

### Hooks

Pass [`HookFns`](https://docs.rs/suitecase/latest/suitecase/suite/struct.HookFns.html) as the last argument to [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html). Each field is `Option<fn(&mut S)>` — [`Some(...)]` or [`None`] / [`HookFns::default()`].

### One line per case in `cargo test` (`test_suite!`)

[`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html) expands to one `#[test]` per listed case. Each test locks a **shared** `Mutex<S>` (keyed by a `static` name you choose), then calls [`run`] with [`RunConfig::filter`] for that case. Later tests can observe mutations from earlier runs on the same suite value.

```rust
use suitecase::{cases, test_suite, Case, HookFns};

// MY_CASES, MY_HOOKS, Counter …

test_suite!(
    Counter,
    MY_SHARED_SUITE,
    Counter::default(),
    MY_CASES,
    MY_HOOKS,
    [test_inc, test_inc_verify]
);
```

Run: `cargo test`. See the macro’s rustdoc for **ordering** (parallel harness) vs a single [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html) with [`RunConfig::all`](https://docs.rs/suitecase/latest/suitecase/suite/struct.RunConfig.html#method.all).

---

## Examples

| Example | Run |
|--------|-----|
| [`examples/quickstart.rs`](examples/quickstart.rs) | `cargo test --example quickstart` |
| [`examples/sqlx_sqlite.rs`](examples/sqlx_sqlite.rs) | `cargo test --example sqlx_sqlite` |
| Every example at once | `cargo test --examples` |
| Integration tests ([`tests/suite.rs`](tests/suite.rs)) | `cargo test` |

The **sqlx** example uses **sqlx** + **tokio**, embedded migrations in `setup_suite`, and [`cases!`](https://docs.rs/suitecase/latest/suitecase/macro.cases.html) blocks that call helper functions, plus [`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html).

```toml
[dependencies]
suitecase = "0.1"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

---

## More documentation

- **`cargo doc --open`** — [`cases!`](https://docs.rs/suitecase/latest/suitecase/macro.cases.html) and [`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html) include declaration / description / examples.
- Crate root and [`suite`](https://docs.rs/suitecase/latest/suitecase/suite/index.html) describe how [`run`](https://docs.rs/suitecase/latest/suitecase/suite/fn.run.html) selects cases and orders hooks.
