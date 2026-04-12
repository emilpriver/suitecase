# testify `assert` → Rust conversion todos (v1.11.1)

This file tracks how each **package-level** assertion from [`github.com/stretchr/testify/assert` v1.11.1](https://pkg.go.dev/github.com/stretchr/testify@v1.11.1/assert) could be expressed in Rust within **this project** (`suitecase`) or as companion APIs/macros.

**Conventions used below**

- **Std**: `assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!*`, `matches!`, `panic!` with `format!` for messages.
- **Pretty diffs**: consider the [`pretty_assertions`](https://docs.rs/pretty_assertions) dev-dependency for struct/collection equality (similar to testify’s output).
- **Floats**: use [`approx`](https://docs.rs/approx) or explicit epsilon/delta checks; never `==` on `f32`/`f64`.
- **Errors**: `Result` + `?` in tests; `assert!(err.is_none())` / `assert!(matches!(err, ...))`; `std::error::Error::downcast_ref` / `source()` chain for “error as” style checks.
- **Async / polling**: `tokio::time::sleep` + loop with timeout, or `tokio::time::timeout` + retry; map to `Eventually` semantics.
- **Suitecase fit**: prefer small **macros** or **helper fns** that take a label/`&str` case name where useful, and compose with existing `suitecase::run` / `suitecase::cases!` without a custom harness.

---

## Equality and sameness

| Go API | Conversion todo |
|--------|-----------------|
| `Equal` / `Equalf` | Add or document a thin wrapper around `assert_eq!(expected, actual)` (or `pretty_assertions::assert_eq!`) with optional `format!`-style message via `assert!(a == b, "…{}…", …)` if messages must match testify’s ergonomics. |
| `EqualValues` / `EqualValuesf` | In Rust, coercion is explicit: todo — add helpers that compare after casting (e.g. `i32` vs `i64`) using `Into`/`as` and then `assert_eq!`, or document pattern `assert_eq!(a as i64, b)`. |
| `EqualExportedValues` / `EqualExportedValuesf` | Rust has no exported/unexported split like Go; todo — for structs, compare via `PartialEq` on a **DTO** with only public fields, or derive equality on a test-only `Record` type that mirrors public data. |
| `Exactly` / `Exactlyf` | Use `assert_eq!` with identical types; if types differ, `assert!(false, "expected same type …")` or compile-time `assert_eq_type` via a macro that fails type check. |
| `NotEqual` / `NotEqualf` | `assert_ne!(a, b)` or `pretty_assertions::assert_ne!` with optional formatted panic message. |
| `NotEqualValues` / `NotEqualValuesf` | Same as `EqualValues` but assert inequality after explicit conversion. |
| `Same` / `Samef` | Pointer identity in Rust: `std::ptr::eq` for references, or `Arc::ptr_eq` / `Rc::ptr_eq`; todo — helper `assert_same_ptr(a, b)` for `&T`. |
| `NotSame` / `NotSamef` | Negation of the above; `assert!(!std::ptr::eq(a, b))`. |

## Booleans

| Go API | Conversion todo |
|--------|-----------------|
| `True` / `Truef` | `assert!(value, …)` or `assert_eq!(value, true)` with message. |
| `False` / `Falsef` | `assert!(!value, …)` or `assert_eq!(value, false)`. |

## Nil / empty / zero

| Go API | Conversion todo |
|--------|-----------------|
| `Nil` / `Nilf` | For `Option`: `assert!(x.is_none())`; for pointers: `assert!(x.is_null())` (raw) or `assert!(x.is_none())` for `Option<Box<T>>`. |
| `NotNil` / `NotNilf` | `assert!(x.is_some())` / `assert!(!x.is_null())` as appropriate. |
| `Empty` / `Emptyf` | `assert!(s.is_empty())` for strings/slices/Vec/HashMap; document that Go’s “empty array” semantics differ — use explicit length or iterator checks. |
| `NotEmpty` / `NotEmptyf` | `assert!(!s.is_empty())`. |
| `Zero` / `Zerof` | `assert_eq!(x, T::default())` or `assert_eq!(x, 0)` for numerics; use trait `num_traits::Zero` only if you add the dependency. |
| `NotZero` / `NotZerof` | `assert_ne!(x, T::default())`. |

## Errors

| Go API | Conversion todo |
|--------|-----------------|
| `NoError` / `NoErrorf` | `result.expect("…")` or `assert!(result.is_ok(), "{:?}", result)` / `unwrap()` in tests. |
| `Error` / `Errorf` | `assert!(result.is_err())` or `assert!(foo().is_err())`. |
| `EqualError` / `EqualErrorf` | `assert_eq!(err.to_string(), expected)` or compare `Display`/`Debug` as needed; for `io::Error`, compare `kind()` or structured variants. |
| `ErrorContains` / `ErrorContainsf` | `assert!(err.to_string().contains(substr))` or chain `source()` loop for wrapped errors. |
| `ErrorIs` / `ErrorIsf` | `assert!(matches!(err, E::Target))` or `std::error::Error::is::<E>()` where applicable; with `anyhow`/`eyre`, use `downcast_ref`. |
| `ErrorAs` / `ErrorAsf` | `assert!(err.downcast_ref::<T>().is_some())` (boxed errors) or match on concrete enum variant. |
| `NotErrorIs` / `NotErrorIsf` | Negate `ErrorIs` checks. |
| `NotErrorAs` / `NotErrorAsf` | Negate `ErrorAs` checks. |

## Strings, JSON, YAML

| Go API | Conversion todo |
|--------|-----------------|
| `Regexp` / `Regexpf` | `assert!(regex::Regex::new(p).unwrap().is_match(s))` — add `regex` as dev-dependency if not present. |
| `NotRegexp` / `NotRegexpf` | Negation of the above. |
| `JSONEq` / `JSONEqf` | Parse both strings to `serde_json::Value` and `assert_eq!` (canonical equality); or compare sorted pretty-print if order-insensitive — document choice. |
| `YAMLEq` / `YAMLEqf` | Same with `serde_yaml` (dev-dep) into `serde_json::Value` or a struct with `PartialEq`. |

## Collections

| Go API | Conversion todo |
|--------|-----------------|
| `Contains` / `Containsf` | `assert!(slice.contains(&x))`, `assert!(s.contains(substr))`, `assert!(map.contains_key(&k))`. |
| `NotContains` / `NotContainsf` | Negate the above. |
| `Subset` / `Subsetf` | For sets: `assert!(subset.iter().all(|x| set.contains(x)))`; for maps, align Go’s key/value rules with `HashMap` subset checks. |
| `NotSubset` / `NotSubsetf` | Negation of subset logic. |
| `ElementsMatch` / `ElementsMatchf` | Sort or use multiset (`HashMap<T, usize>` count) and `assert_eq!`; todo — helper `assert_same_elements(a, b)`. |
| `NotElementsMatch` / `NotElementsMatchf` | Negation of multiset equality. |
| `Len` / `Lenf` | `assert_eq!(collection.len(), n)`. |

## Ordering and numeric traits

| Go API | Conversion todo |
|--------|-----------------|
| `Greater` / `Greaterf` | `assert!(a > b)` with same-type comparisons. |
| `GreaterOrEqual` / `GreaterOrEqualf` | `assert!(a >= b)`. |
| `Less` / `Lessf` | `assert!(a < b)`. |
| `LessOrEqual` / `LessOrEqualf` | `assert!(a <= b)`. |
| `Positive` / `Positivef` | `assert!(x > T::zero())` or `assert!(x > 0)` for signed types. |
| `Negative` / `Negativef` | `assert!(x < 0)`. |
| `IsIncreasing` / `IsIncreasingf` | `windows(2).all(|w| w[0] < w[1])` on slices. |
| `IsDecreasing` / `IsDecreasingf` | `windows(2).all(|w| w[0] > w[1])`. |
| `IsNonDecreasing` / `IsNonDecreasingf` | `<=` chain. |
| `IsNonIncreasing` / `IsNonIncreasingf` | `>=` chain. |

## Floating-point

| Go API | Conversion todo |
|--------|-----------------|
| `InDelta` / `InDeltaf` | `(a - b).abs() <= delta` with appropriate type, or `approx::abs_diff_eq!` (dev-dep). |
| `InDeltaSlice` / `InDeltaSlicef` | Zip slices and assert each pair in delta, plus same length. |
| `InDeltaMapValues` / `InDeltaMapValuesf` | Align keys and compare values in delta. |
| `InEpsilon` / `InEpsilonf` | Relative epsilon compare: `((a - b).abs() <= epsilon * max(a.abs(), b.abs()))` or `approx` traits. |
| `InEpsilonSlice` / `InEpsilonSlicef` | Per-element epsilon like delta slice. |

## Types and interfaces

| Go API | Conversion todo |
|--------|-----------------|
| `IsType` / `IsTypef` | `assert!(value.is::<T>())` for `dyn Any`, or compile-time type checks via generics in helpers. |
| `IsNotType` / `IsNotTypef` | Negation. |
| `Implements` / `Implementsf` | Rust: trait bounds are compile-time; runtime “implements” is uncommon — todo — document using trait objects (`as_ref()` to `dyn Trait`) or compile-only tests with `fn assert_impl<T: Trait>() {}`. |
| `NotImplements` / `NotImplementsf` | Usually compile-time (negative trait bounds / lack of impl); document that Rust differs from Go here. |

## Time

| Go API | Conversion todo |
|--------|-----------------|
| `WithinDuration` / `WithinDurationf` | `assert!((a - b).abs() <= delta)` with `std::time::Duration` or `chrono::Duration`. |
| `WithinRange` / `WithinRangef` | `assert!(start <= actual && actual <= end)`. |

## Filesystem

| Go API | Conversion todo |
|--------|-----------------|
| `FileExists` / `FileExistsf` | `assert!(std::path::Path::new(p).is_file())`. |
| `NoFileExists` / `NoFileExistsf` | `assert!(!path.is_file())` (and clarify symlink behavior). |
| `DirExists` / `DirExistsf` | `assert!(path.is_dir())`. |
| `NoDirExists` / `NoDirExistsf` | `assert!(!path.is_dir())`. |

## HTTP handlers

| Go API | Conversion todo |
|--------|-----------------|
| `HTTPBody` | In Rust tests, call `hyper`/`reqwest`/`axum::test` helpers to get body bytes/string; no direct analog — todo — document one recommended pattern (e.g. `tower::Service::call` + body aggregate). |
| `HTTPSuccess` / `HTTPSuccessf` | Assert status `200..299` on `Response`. |
| `HTTPError` / `HTTPErrorf` | Assert `400..599` or `>= 400` as needed. |
| `HTTPRedirect` / `HTTPRedirectf` | Status `301/302/303/307/308` + optional `Location` header check. |
| `HTTPStatusCode` / `HTTPStatusCodef` | `assert_eq!(response.status(), StatusCode::…)`. |
| `HTTPBodyContains` / `HTTPBodyContainsf` | String/bytes substring on body after read. |
| `HTTPBodyNotContains` / `HTTPBodyNotContainsf` | Negation. |

## Async / polling

| Go API | Conversion todo |
|--------|-----------------|
| `Eventually` / `Eventuallyf` | Loop with `std::thread::sleep` + deadline, or `tokio::time::timeout` + retry; todo — optional macro `eventually!(deadline, tick, \|\| cond)` for async tests. |
| `EventuallyWithT` / `EventuallyWithTf` | Collect failures per tick: aggregate `Vec<String>` or use `miette`/custom collector; map `CollectT` to a small struct that implements `Write`/`push_str` for errors. |
| `Never` / `Neverf` | Assert condition stays false until deadline (inverse of Eventually). |

## Panics

| Go API | Conversion todo |
|--------|-----------------|
| `Panics` / `Panicsf` | `std::panic::catch_unwind` + `assert!(result.is_err())` or use `assert_panic` pattern / third-party `assert-panic` crate. |
| `NotPanics` / `NotPanicsf` | `catch_unwind` and assert unwind did not happen. |
| `PanicsWithValue` / `PanicsWithValuef` | Downcast panic payload to `&str`/`String` and compare (fragile — document limitations). |
| `PanicsWithError` / `PanicsWithErrorf` | If panic is `String`/`&str`, compare; for `anyhow!`, match message substring. |

## Custom / control flow

| Go API | Conversion todo |
|--------|-----------------|
| `Condition` / `Conditionf` | `assert!(closure(), "…")` where `closure: FnOnce() -> bool`. |
| `Fail` / `Failf` | `panic!("…")` or `unreachable!` with message. |
| `FailNow` / `FailNowf` | In Rust tests, `panic!` aborts the test; for “continue other cases” semantics, use suitecase’s isolation — document no direct `FailNow` in one `#[test]` other than panic. |

## Helpers (non-`TestingT`)

| Go API | Conversion todo |
|--------|-----------------|
| `CallerInfo` | `std::panic::Location::caller()` or `backtrace` crate for stack; mostly for custom failure formatting — low priority unless building a testify-like printer. |
| `ObjectsAreEqual` | Use `PartialEq` or `==` in Rust; wrap as `fn objects_are_equal<T: PartialEq>(a: &T, b: &T) -> bool`. |
| `ObjectsAreEqualValues` | Same as `EqualValues` — explicit conversion then `==`. |
| `ObjectsExportedFieldsAreEqual` | Same as `EqualExportedValues` — compare public projection types. |

## Constructor

| Go API | Conversion todo |
|--------|-----------------|
| `New(t)` | In Rust, no `*testing.T`; use a struct holding context: e.g. `struct AssertCx { label: &'static str }` with methods that panic with label, or rely on module path in `panic!`. Optional: integrate with `tracing` spans per case name from suitecase’s `Case::name`. |

---

## Types (supporting APIs)

| Go type | Conversion todo |
|---------|-----------------|
| `Assertions` | Rust struct with `&'static str` or `&dyn Fn()` context for messages; methods mirror free functions (builder or trait object). |
| `TestingT` | Replace with `&mut dyn TestContext` that only needs `fail(&str)` / `fail_fmt` or use `panic!` + `format!` everywhere. |
| `Comparison` | `FnOnce() -> bool` or `FnMut` in Rust. |
| `CollectT` | Small collector type: `Vec<String>` + `fn push_error(&mut self, msg: String)` for Eventually-style polling. |
| `PanicTestFunc` | `FnOnce()` in Rust. |
| `*AssertionFunc` typedefs | Function pointers or `macro_rules!` to reduce boilerplate in table-driven tests. |
| `AnError` | `std::io::Error::new(ErrorKind::Other, "test")` or a crate-local `static TEST_ERR: Lazy<io::Error>`. |

---

## Meta

- [ ] Decide whether these live in **`suitecase`** as an `assert` module, a separate **`suitecase-assert`** crate, or remain **documentation-only** patterns.
- [ ] Add **dev-dependencies** only where needed (`pretty_assertions`, `regex`, `serde_json`, `approx`, etc.).
- [ ] Keep **edition 2024** and **minimal dependencies** in `[dependencies]`; prefer std + explicit test helpers.
