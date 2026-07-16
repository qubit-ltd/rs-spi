# rs-spi Direct Error Matching Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace parallel resolution/attempt kind APIs with direct non-exhaustive enum matching, centralize decisive-attempt selection, migrate `qubit-mime`, and document the selector benchmark prerequisite.

**Architecture:** `ResolutionError` and `AttemptFailure` remain the single structured error representation. Cross-variant resolution semantics stay as query methods, while variant-specific context is obtained by matching the enum directly; `decisive_attempt` owns the shared terminal-cause rule used by MIME adapters.

**Tech Stack:** Rust 2024, `thiserror` 2.0, external integration tests, repository CI scripts.

## Global Constraints

- Breaking changes are authorized because `qubit-spi` is unpublished.
- Delete `ResolutionErrorKind`, `AttemptFailureKind`, and their dedicated tests; add no compatibility aliases.
- Keep `ResolutionError::{attempts, termination, terminal_attempt, is_absence}`.
- Do not implement selector allocation optimization or add a benchmark in this change.
- Add the exact benchmark prerequisite to Rustdoc on `ProviderSelector::parse` and `ProviderResolver::create_named`.
- Keep all tests under mirrored `tests/` paths; add no inline tests.
- Run repository validation in the order `align-ci.sh`, then `ci-check.sh`, and run `coverage.sh json` only if CI reports coverage below threshold.
- Do not run `git add`, `git commit`, or `git push`.

---

### Task 1: Decisive-attempt contract

**Files:**
- Modify: `tests/provider_resolver_tests.rs`
- Modify: `src/error/resolution_error.rs`

**Interfaces:**
- Consumes: `ResolutionTermination::{Exhausted, StoppedByPolicy}` and ordered `AttemptFailure` storage.
- Produces: `pub fn decisive_attempt(&self) -> Option<&AttemptFailure>`.

- [ ] **Step 1: Write the failing contract test**

Extend the existing policy-stop and aggregate tests to assert:

```rust
assert_eq!(
    error.terminal_attempt().map(ToString::to_string),
    error.decisive_attempt().map(ToString::to_string),
);
```

For singleton exhaustion, assert `decisive_attempt().is_some()`. For
multi-attempt exhaustion and `ResolutionError::EmptyRegistry`, assert
`decisive_attempt().is_none()`.

- [ ] **Step 2: Run the focused test and verify RED**

Run from `rs-spi`:

```bash
cargo test --test integration_tests provider_resolver
```

Expected: compilation fails because `ResolutionError::decisive_attempt` does
not exist.

- [ ] **Step 3: Implement the minimal method**

Add after `terminal_attempt`:

```rust
/// Returns the attempt that directly explains the aggregate outcome.
///
/// # Returns
///
/// The terminal attempt after a policy stop, the only attempt after singleton
/// exhaustion, or `None` for non-aggregate errors and ambiguous multi-attempt
/// exhaustion.
#[inline]
#[must_use]
pub fn decisive_attempt(&self) -> Option<&AttemptFailure> {
    match self {
        Self::NoProviderSucceeded {
            attempts,
            termination: ResolutionTermination::StoppedByPolicy,
        } => attempts.last(),
        Self::NoProviderSucceeded {
            attempts,
            termination: ResolutionTermination::Exhausted,
        } => match attempts.as_ref() {
            [attempt] => Some(attempt),
            _ => None,
        },
        _ => None,
    }
}
```

- [ ] **Step 4: Run the focused test and verify GREEN**

```bash
cargo test --test integration_tests provider_resolver
```

Expected: all `provider_resolver` tests pass without warnings.

### Task 2: Remove parallel kind APIs and migrate core tests

**Files:**
- Delete: `src/error/attempt_failure_kind.rs`
- Delete: `src/error/resolution_error_kind.rs`
- Delete: `tests/error/attempt_failure_kind_tests.rs`
- Delete: `tests/error/resolution_error_kind_tests.rs`
- Modify: `src/error/mod.rs`
- Modify: `src/error/attempt_failure.rs`
- Modify: `src/error/resolution_error.rs`
- Modify: `tests/error/mod.rs`
- Modify: `tests/error/attempt_failure_tests.rs`
- Modify: `tests/error/resolution_error_tests.rs`
- Modify: `tests/provider_resolver_tests.rs`
- Modify: `tests/provider_selection_tests.rs`

**Interfaces:**
- Removes: `AttemptFailureKind`, `AttemptFailure::kind`,
  `AttemptFailure::{requested_selector, provider_id, provider_error}`.
- Removes: `ResolutionErrorKind`, `ResolutionError::kind`,
  `ResolutionError::{invalid_selector_input, invalid_selector_index,
  selector_error, unknown_selector}`.
- Preserves: direct public matching on both `#[non_exhaustive]` enums and all
  aggregate/general resolution queries.

- [ ] **Step 1: Rewrite focused tests around direct matching**

Use variant patterns to assert correlated fields. For example:

```rust
let AttemptFailure::ProviderError {
    requested_selector,
    provider_id,
    error,
} = attempt
else {
    panic!("named resolution should retain a provider failure");
};
assert_eq!(
    Some("file-command"),
    requested_selector.as_ref().map(ProviderSelector::as_str),
);
assert_eq!("file-command", provider_id.as_str());
assert_eq!(ProviderErrorKind::Unavailable, error.kind());
```

Remove assertions that merely duplicate a successfully matched variant through
`kind()` or optional context accessors.

- [ ] **Step 2: Remove modules, exports, methods, and dedicated tests**

Delete the four dedicated files. Remove their `mod`/`pub use` entries and the
two test-module declarations. Remove only the parallel methods listed in the
Interfaces section; retain all general queries and formatting/error-source
behavior.

- [ ] **Step 3: Verify the complete rs-spi test target**

```bash
cargo test --test integration_tests
```

Expected: the integration target compiles without either removed kind type and
all tests pass.

### Task 3: Migrate MIME error adapters to direct matching

**Files:**
- Modify: `../rs-mime/src/detector/mime_detector_registry.rs`
- Modify: `../rs-mime/src/classifier/media_stream_classifier_registry.rs`

**Interfaces:**
- Consumes: `ResolutionError::decisive_attempt` and direct patterns for
  `ResolutionError` and `AttemptFailure`.
- Produces: the existing `MimeError` classifications without correlated-field
  `expect` calls.

- [ ] **Step 1: Convert detector resolution mapping**

Match `ResolutionError` by reference so the decisive-attempt query can still
borrow the complete error:

```rust
match &error {
    ResolutionError::InvalidSelector { input, source, .. } => { /* existing mapping */ }
    ResolutionError::UnknownProvider { selector } => { /* existing mapping */ }
    ResolutionError::EmptySelection | ResolutionError::EmptyRegistry => { /* existing mapping */ }
    ResolutionError::NoProviderSucceeded { .. } => error
        .decisive_attempt()
        .map(detector_attempt_error)
        .unwrap_or(MimeError::NoAvailableDetector { reason: message }),
    _ => MimeError::NoAvailableDetector { reason: message },
}
```

Match `AttemptFailure::{UnknownProvider, ProviderError}` directly in
`detector_attempt_error`, retaining a wildcard fallback for future variants.

- [ ] **Step 2: Convert classifier resolution mapping**

Use the same direct `ResolutionError` structure. Match
`AttemptFailure::ProviderError { provider_id, error, .. }` directly; map any
other attempt to `MimeError::NoAvailableClassifier` without panic.

- [ ] **Step 3: Verify MIME tests**

Run from `rs-mime`:

```bash
cargo test --test integration_tests
```

Expected: all MIME integration tests pass and `rg` finds no removed SPI kind
types or correlated-field `expect` messages in `src/`.

### Task 4: Add selector benchmark prerequisite to Rustdoc

**Files:**
- Modify: `src/provider_selector.rs`
- Modify: `src/provider_resolver.rs`

**Interfaces:**
- Produces: documentation only; lookup behavior and signatures remain unchanged.

- [ ] **Step 1: Add the exact Rustdoc note**

Add this paragraph to both `ProviderSelector::parse` and
`ProviderResolver::create_named`:

```rust
/// # Performance
///
/// TODO: Before adding a no-allocation fast path, benchmark representative
/// repeated canonical-selector lookups, including filesystem URI schemes, and
/// retain the optimization only when the measurements show a material benefit.
```

- [ ] **Step 2: Confirm documentation scope**

```bash
rg -n -A5 '# Performance' src/provider_selector.rs src/provider_resolver.rs
```

Expected: exactly two benchmark-prerequisite sections and no implementation
change in either function body.

### Task 5: Repository-prescribed verification

**Files:**
- Inspect: all modified files in `rs-spi` and `rs-mime`
- Verify: `rs-fs` and `rs-magika` as unchanged direct dependents

**Interfaces:**
- Consumes: final source and test changes.
- Produces: fresh formatting, lint, build, test, documentation, and downstream
  compatibility evidence.

- [ ] **Step 1: Verify no removed API remains**

From `rust-common`:

```bash
rg -n 'ResolutionErrorKind|AttemptFailureKind|invalid_selector_input|invalid_selector_index|unknown_selector\(|requested_selector\(|provider_error\(' rs-spi/src rs-spi/tests rs-mime/src rs-mime/tests
```

Expected: no matches associated with the removed resolution/attempt APIs.

- [ ] **Step 2: Run rs-spi validation in order**

```bash
./align-ci.sh
./ci-check.sh
```

Run from `rs-spi`. If CI alone reports coverage below threshold, run:

```bash
./coverage.sh json
```

- [ ] **Step 3: Run direct-dependent validation in order**

Run `./align-ci.sh` followed by `./ci-check.sh` from each repository root in
this order: `rs-fs`, `rs-mime`, `rs-magika`. Apply the same conditional coverage
rule independently to each repository.

- [ ] **Step 4: Inspect final diffs and requirements**

```bash
git -C rs-spi --no-pager diff --check
git -C rs-spi --no-pager diff
git -C rs-mime --no-pager diff --check
git -C rs-mime --no-pager diff
git -C rs-fs status --short
git -C rs-magika status --short
```

Expected: no whitespace errors, no unrelated modifications, no selector
optimization, and no changes in `rs-fs` or `rs-magika` unless verification
proved a source migration necessary.
