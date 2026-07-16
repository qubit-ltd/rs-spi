# rs-spi Resolution Outcome Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Preserve resolution termination semantics, provide stable error query APIs, centralize selection parsing, and migrate direct consumers to validated selections.

**Architecture:** `ResolutionError` retains ordered attempts plus a typed termination reason. Non-exhaustive errors expose observation methods, resolver raw methods delegate through `ProviderSelection`, and `MimeConfig` validates and stores selections during loading.

**Tech Stack:** Rust 2024, `thiserror` 2.0, external integration tests, repository CI scripts.

## Global Constraints

- Breaking API changes are authorized; do not add compatibility aliases.
- Do not implement selector allocation optimization.
- Keep one Rust type per snake-case file and tests under mirrored `tests/` paths.
- Preserve existing configuration keys and textual values.
- Use TDD and observe every new contract fail before production edits.
- Do not run `git add`, `git commit`, or `git push`.

---

### Task 1: Resolution termination contract

**Files:**
- Create: `src/resolution_termination.rs`
- Create: `src/error/resolution_error_kind.rs`
- Create: `src/error/attempt_failure_kind.rs`
- Modify: `src/error/resolution_error.rs`
- Modify: `src/error/attempt_failure.rs`
- Modify: `src/error/mod.rs`
- Modify: `src/lib.rs`
- Test: `tests/error/resolution_error_tests.rs`
- Test: `tests/error/attempt_failure_tests.rs`
- Test: `tests/provider_resolver_tests.rs`

**Interfaces:**
- Produces: `ResolutionTermination::{Exhausted, StoppedByPolicy}`.
- Produces: non-exhaustive `ResolutionErrorKind` and `AttemptFailureKind`.
- Produces: stable query methods documented in the design spec.

- [ ] **Step 1: Write failing termination tests**

Add resolver tests that assert an `OnAbsence` chain ending on an
`InitializationFailed` provider reports:

```rust
assert_eq!(
    Some(ResolutionTermination::StoppedByPolicy),
    error.termination(),
);
assert_eq!(
    Some(ProviderErrorKind::InitializationFailed),
    error
        .terminal_attempt()
        .and_then(AttemptFailure::provider_error)
        .map(ProviderError::kind),
);
```

Add an `OnAnyError` exhausted chain assertion for
`ResolutionTermination::Exhausted`, plus focused tests for every query method.

- [ ] **Step 2: Verify RED**

Run from `rs-spi`:

```bash
cargo test --test integration_tests resolution_termination
```

Expected: compilation fails because `ResolutionTermination` and the query
methods do not exist.

- [ ] **Step 3: Implement minimal error contract**

Add the three enums in separate files. Change the aggregate variant to:

```rust
NoProviderSucceeded {
    attempts: Box<[AttemptFailure]>,
    termination: ResolutionTermination,
}
```

Add crate-private `exhausted(attempts)` and
`stopped_by_policy(attempts)` constructors, then implement the exact query APIs
from the design spec. Mark all public error enums `#[non_exhaustive]`.

- [ ] **Step 4: Update resolver construction and verify GREEN**

Use `stopped_by_policy` only at policy rejection branches and `exhausted` at
normal candidate exhaustion or named failure. Run:

```bash
cargo test --test integration_tests resolution_termination
cargo test --test integration_tests resolution_error
cargo test --test integration_tests attempt_failure
```

Expected: all focused tests pass.

### Task 2: Stable registration and selector error observation

**Files:**
- Modify: `src/error/registration_error.rs`
- Modify: `src/error/provider_selector_error.rs`
- Test: `tests/error/registration_error_tests.rs`
- Test: `tests/provider_selector_tests.rs`

**Interfaces:**
- Produces: `RegistrationError::{selector, existing_provider, provider}`.
- Produces: `ProviderSelectorError::{input, normalized, is_empty}`.

- [ ] **Step 1: Add accessor tests and verify RED**

Use the accessors without enum destructuring, then run:

```bash
cargo test --test integration_tests registration_error
cargo test --test integration_tests provider_selector
```

Expected: compilation fails on missing methods.

- [ ] **Step 2: Implement thin accessors and verify GREEN**

Implement `#[inline(always)]` accessors returning borrowed fields and run the
same commands. Expected: focused tests pass.

### Task 3: Centralize validated selection parsing

**Files:**
- Modify: `src/error/resolution_error.rs`
- Modify: `src/provider_resolver.rs`
- Modify: `src/provider_descriptor.rs`
- Test: `tests/provider_resolver_tests.rs`

**Interfaces:**
- Produces: internal conversion from `ProviderSelectionError` to
  `ResolutionError`.
- Preserves: raw named/chain invalid input and selector indexes.

- [ ] **Step 1: Strengthen raw/prevalidated equivalence tests**

Assert raw and prevalidated named/chain paths select the same canonical
provider and raw invalid chains preserve their index. Temporarily change the
expected conversion path so the focused test fails before refactoring.

- [ ] **Step 2: Verify RED**

```bash
cargo test --test integration_tests raw_selection
```

Expected: the new conversion contract is missing.

- [ ] **Step 3: Delegate raw methods through `ProviderSelection`**

Convert `ProviderSelectionError::{InvalidSelector, EmptyChain}` into the
corresponding resolution errors. Implement `create_named` and `create_chain`
as construction followed by `self.create(&selection, config)`.

Reorder `ProviderDescriptor` methods to `new`, `with_aliases`, `with_priority`,
`id`, `aliases`, `priority`, moving complete Rustdoc and attributes.

- [ ] **Step 4: Verify GREEN**

```bash
cargo test --test integration_tests provider_resolver
cargo test --test integration_tests provider_selection
```

Expected: all resolver and selection tests pass.

### Task 4: Store validated selections in MIME configuration

**Files:**
- Modify: `../rs-mime/src/mime_config.rs`
- Modify: `../rs-mime/src/detector/mime_detector_registry.rs`
- Modify: `../rs-mime/src/classifier/media_stream_classifier_registry.rs`
- Modify: `../rs-mime/tests/api/mime_config_tests.rs`
- Modify: `../rs-mime/tests/detector/mime_detector_registry_tests.rs`
- Modify: `../rs-mime/tests/classifier/media_stream_classifier_registry_tests.rs`

**Interfaces:**
- Produces: `MimeConfig::mime_detector_selection()`.
- Produces: `MimeConfig::media_stream_classifier_selection()`.
- Removes: raw default/fallback selector getters.

- [ ] **Step 1: Add configuration validation and termination mapping tests**

Add tests proving invalid selectors fail in `MimeConfig::from_config`, valid
fallbacks become a chain, `auto` becomes automatic selection, and a
multi-attempt policy stop maps the terminal provider failure precisely.

- [ ] **Step 2: Verify RED**

```bash
cargo test --test integration_tests mime_config_selection
cargo test --test integration_tests policy_stop
```

Expected: missing typed-selection methods and imprecise aggregate mapping.

- [ ] **Step 3: Implement typed config and migrate registries**

Parse detector primary/fallback values and classifier values during config
construction. Store `ProviderSelection`, expose borrowed getters, and have both
registries call `resolver.create(selection, config)`. Rewrite error adapters to
use kinds and accessors, using `terminal_attempt()` for policy stops.

- [ ] **Step 4: Verify GREEN**

Run the two focused commands again. Expected: all focused MIME tests pass.

### Task 5: Migrate remaining direct consumers and verify repositories

**Files:**
- Modify: `../rs-fs/src/provider/file_system_registry.rs`
- Modify: affected `../rs-fs/tests/**`
- Modify: affected `../rs-magika/**` only when compilation requires it
- Modify: `README.md` and `README.zh_CN.md`

**Interfaces:**
- Consumes: stable error query APIs.
- Produces: no downstream exhaustive match on `qubit-spi` error layout.

- [ ] **Step 1: Migrate filesystem mapping**

Replace structural matching with `ResolutionError::is_absence()` and kind-based
fallback mapping. Add a focused regression test for absence classification.

- [ ] **Step 2: Update documentation**

Document termination semantics, non-exhaustive errors, query APIs, and
configuration-time selection validation in both rs-spi READMEs.

- [ ] **Step 3: Run repository-prescribed verification in order**

From each repository root, in this order: `rs-spi`, `rs-fs`, `rs-mime`,
`rs-magika`:

```bash
./align-ci.sh
./ci-check.sh
```

If and only if CI reports coverage below threshold, run:

```bash
./coverage.sh json
```

Inspect every alignment change and fix only in-scope failures, then rerun the
affected repository sequence.

- [ ] **Step 4: Review final diff and requirements**

Confirm no selector allocation optimization, compatibility alias, inline test,
or unrelated refactor entered the changes. Confirm all new types have their
own files, complete Rustdoc, and external tests.
