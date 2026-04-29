# Changelog

## 0.0.8 — 2026-04-29

### Changed

* Changed to `suitecase test` command for running tests ([#15](https://github.com/emilpriver/suitecase/pull/15))
* Added support for workspace and release flags to test command ([#15](https://github.com/emilpriver/suitecase/pull/15))
* Added new CLI for running tests with multiple output formats ([#14](https://github.com/emilpriver/suitecase/pull/14))
* Added suite name to test output ([#14](https://github.com/emilpriver/suitecase/pull/14))

### Removed

* Removed running tests using test macro, replaced with CLI ([#14](https://github.com/emilpriver/suitecase/pull/14))

## 0.0.5 — 2026-04-12

### Added

* New [`suitecase::mock`](https://docs.rs/suitecase/latest/suitecase/mock/index.html) module: [testify/mock](https://pkg.go.dev/github.com/stretchr/testify/mock)-style **expectations** and **call recording** on a thread-safe [`Mock`](https://docs.rs/suitecase/latest/suitecase/mock/struct.Mock.html), with [`mock_args!`](https://docs.rs/suitecase/latest/suitecase/macro.mock_args.html), matchers (`eq`, `anything`, `matched_by`, …), and [`TestingT`](https://docs.rs/suitecase/latest/suitecase/mock/trait.TestingT.html) for verification.  #11
* [`examples/mock.rs`](examples/mock.rs) showing how to wire a mock into tests.

### Changed

* README: **Mocking** section and table-of-contents link; **AI-assisted changes** points to [`AI_USAGE.md`](AI_USAGE.md), which documents expectations for AI-assisted contributions.

## 0.0.4 — 2026-04-12

### Added

* Introduced the [`suitecase::assert`](https://docs.rs/suitecase/latest/suitecase/assert/index.html) module: testify-style panicking helpers for tests (`equal`, `contains`, error chain checks, float deltas/epsilons, `Option`/`Result`, ordering, filesystem, panic expectations, pointers, durations, and more), with [`#[track_caller]`](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-track_caller-attribute) so failures point at the call site.  #8
* Documented an **inline** [`test_suite!`](https://docs.rs/suitecase/latest/suitecase/macro.test_suite.html) form so case bodies and names are written once (same shape as [`cases!`](https://docs.rs/suitecase/latest/suitecase/macro.cases.html)), avoiding duplicate case lists.  #8

### Changed

* Set `documentation`, `homepage`, and `repository` in `Cargo.toml` for docs.rs and crates.io.  #8
* Updated the `quickstart` and `sqlx_sqlite` examples for the new APIs and patterns.  #8
* Renamed the release workflow job in `.github/workflows/release.yml`.  #8
