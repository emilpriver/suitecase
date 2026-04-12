# Changelog

## 0.0.4 — 2026-04-12

### Added

* Introduced the [`suitecase::assert`](https://docs.rs/suitecase/latest/suitecase/assert/index.html) module: testify-style panicking helpers for tests (`equal`, `contains`, error chain checks, float deltas/epsilons, `Option`/`Result`, ordering, filesystem, panic expectations, pointers, durations, and more), with [`#[track_caller]`](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-track_caller-attribute) so failures point at the call site.  #8
* Documented an **inline** [`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html) form so case bodies and names are written once (same shape as [`cases!`](https://docs.rs/suitecase/latest/suitecase/macro.cases.html)), avoiding duplicate case lists.  #8

### Changed

* Set `documentation`, `homepage`, and `repository` in `Cargo.toml` for docs.rs and crates.io.  #8
* Updated the `quickstart` and `sqlx_sqlite` examples for the new APIs and patterns.  #8
* Renamed the release workflow job in `.github/workflows/release.yml`.  #8
