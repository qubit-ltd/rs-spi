# rs-spi Error Module Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace opaque resolution diagnostics with directly matchable public enums, expose all SPI errors only through `qubit_spi::error`, remove impossible ID reparsing, align `ProviderDescriptor`, document reusable selections, and migrate every direct workspace consumer.

**Architecture:** `ResolutionError` and `AttemptFailure` become their sole public and stored enum representations. All public error files move below `src/error/`, remaining private error representations move below `src/error/internal/`, and root error re-exports disappear. Direct consumers match enum variants and concrete fields rather than coordinating kind enums with optional accessors.

**Tech Stack:** Rust 2024, `thiserror` 2.0, external integration tests, Rustdoc doctests, repository `align-ci.sh` / `ci-check.sh` / conditional `coverage.sh json`.

## Global Constraints

- Breaking changes are intentional; do not add compatibility aliases or root error re-exports.
- Keep `ProviderSelection` and add a runnable pre-parse-and-reuse Rustdoc example.
- Keep validation and registration errors opaque; only `ResolutionError` and `AttemptFailure` become public enums.
- Every Rust type remains in one snake-case file; private error representations live below `src/error/internal/`.
- Do not modify or revert pre-existing user changes in `rs-mime/Cargo.toml`, `rs-mime/Cargo.lock`, or `rs-magika/Cargo.lock` except unavoidable lockfile updates produced by prescribed repository scripts; inspect any such updates before retaining them.
- Do not run `git add`, `git commit`, or `git push`; the user has not authorized Git writes.
- Use `apply_patch` for source and documentation edits.
- Follow TDD: observe the requested API fail before implementing production code.

---

### Task 1: Specify directly matchable resolution diagnostics

**Files:**
- Modify: `tests/attempt_failure_tests.rs`
- Modify: `tests/resolution_error_tests.rs`
- Modify: `tests/provider_resolver_tests.rs`

**Interfaces:**
- Consumes: current `ProviderResolver`, `ProviderRegistry`, and provider fixtures.
- Produces: executable expectations for `qubit_spi::error::{AttemptFailure, ResolutionError}` and their public variants.

- [ ] **Step 1: Change one attempt test to the desired public enum API**

Replace kind/accessor assertions in `test_attempt_failure_preserves_provider_error_source` with direct destructuring:

```rust
use qubit_spi::error::{
    AttemptFailure,
    ProviderError,
    ProviderErrorKind,
    ResolutionError,
};

let ResolutionError::NoProviderSucceeded { attempts } = error else {
    panic!("one provider failure should produce an aggregate error");
};
let [AttemptFailure::ProviderError {
    requested_selector,
    provider_id,
    error,
}] = attempts.as_ref()
else {
    panic!("one named provider must produce exactly one provider attempt");
};

assert_eq!(
    Some("file-command"),
    requested_selector.as_ref().map(ProviderSelector::as_str),
);
assert_eq!("file-command", provider_id.as_str());
assert_eq!(ProviderErrorKind::Unavailable, error.kind());
assert_eq!("file executable is absent", error.reason());
assert!(std::error::Error::source(error).is_some());
```

- [ ] **Step 2: Run the focused test and verify RED**

Run from `rs-spi`:

```bash
cargo test --test integration_tests test_attempt_failure_preserves_provider_error_source
```

Expected: compilation fails because `qubit_spi::error` is not public and `ResolutionError` / `AttemptFailure` do not expose those enum variants.

- [ ] **Step 3: Specify direct unknown-provider matching**

Update `test_resolution_error_exposes_its_kind_and_attempts` to import `ResolutionError` from `qubit_spi::error` and match:

```rust
let ResolutionError::UnknownProvider { selector } = error else {
    panic!("an empty registry lookup should report an unknown provider");
};
assert_eq!("missing", selector.as_str());
```

Update provider resolver tests so invalid, empty, unknown, and exhausted results match the concrete variants:

```rust
match error {
    ResolutionError::InvalidSelector {
        input,
        selector_index,
        source,
    } => {
        assert_eq!(" Bad Selector ", input.as_ref());
        assert_eq!(None, selector_index);
        assert_eq!(ProviderSelectorErrorKind::Invalid, source.kind());
    }
    other => panic!("expected invalid selector, got {other:?}"),
}
```

Use analogous matches for `EmptySelection`, `EmptyRegistry`, and
`NoProviderSucceeded { attempts }`. Do not retain assertions against removed
kind enums or removed optional accessors.

- [ ] **Step 4: Keep the suite red until production implementation begins**

Run:

```bash
cargo test --test integration_tests resolution_error
```

Expected: compilation still fails only because the desired public enum API is missing, not because of syntax or fixture errors.

### Task 2: Create the public error module and enum representations

**Files:**
- Create: `src/error/mod.rs`
- Create: `src/error/internal/mod.rs`
- Move and modify: `src/attempt_failure.rs` -> `src/error/attempt_failure.rs`
- Move and modify: `src/resolution_error.rs` -> `src/error/resolution_error.rs`
- Move without behavioral redesign: all `provider_*_error*.rs` and `registration_error*.rs` files -> `src/error/`
- Move: remaining `src/internal/*_error_repr.rs` -> `src/error/internal/`
- Delete: `src/attempt_failure_kind.rs`
- Delete: `src/resolution_error_kind.rs`
- Delete: `src/internal/attempt_failure_repr.rs`
- Delete: `src/internal/resolution_error_repr.rs`
- Modify: `src/internal/mod.rs`
- Modify: `src/lib.rs`
- Modify: all `src/*.rs` imports that consume error types

**Interfaces:**
- Consumes: `ProviderSelector`, `ProviderId`, and `ProviderError`.
- Produces: `pub mod error` with one canonical path for all public errors; `AttemptFailure` and `ResolutionError` public enums.

- [ ] **Step 1: Add the public error module declarations and re-exports**

Create `src/error/mod.rs` with the repository header, module Rustdoc, private
file modules, and these public re-exports:

```rust
//! Errors and diagnostics produced by provider validation, registration, and
//! resolution.

mod attempt_failure;
mod internal;
mod provider_descriptor_error;
mod provider_descriptor_error_kind;
mod provider_error;
mod provider_error_kind;
mod provider_id_error;
mod provider_id_error_kind;
mod provider_selection_error;
mod provider_selection_error_kind;
mod provider_selector_error;
mod provider_selector_error_kind;
mod registration_error;
mod registration_error_kind;
mod resolution_error;

pub use attempt_failure::AttemptFailure;
pub use provider_descriptor_error::ProviderDescriptorError;
pub use provider_descriptor_error_kind::ProviderDescriptorErrorKind;
pub use provider_error::ProviderError;
pub use provider_error_kind::ProviderErrorKind;
pub use provider_id_error::ProviderIdError;
pub use provider_id_error_kind::ProviderIdErrorKind;
pub use provider_selection_error::ProviderSelectionError;
pub use provider_selection_error_kind::ProviderSelectionErrorKind;
pub use provider_selector_error::ProviderSelectorError;
pub use provider_selector_error_kind::ProviderSelectorErrorKind;
pub use registration_error::RegistrationError;
pub use registration_error_kind::RegistrationErrorKind;
pub use resolution_error::ResolutionError;
```

Create `src/error/internal/mod.rs` with explicit declarations and crate-private
re-exports for the five remaining representation enums. Do not use wildcard
imports.

- [ ] **Step 2: Replace `AttemptFailure` storage with its public enum**

Implement in `src/error/attempt_failure.rs`:

```rust
#[derive(Clone, Debug)]
pub enum AttemptFailure {
    /// Selector lookup reached no provider.
    UnknownProvider {
        /// Normalized selector retained from the request.
        requested_selector: ProviderSelector,
    },
    /// A provider factory returned a classified error.
    ProviderError {
        /// Explicit selector, or `None` for automatic selection.
        requested_selector: Option<ProviderSelector>,
        /// Canonical provider reached by lookup.
        provider_id: ProviderId,
        /// Original provider error retained with its causal source.
        error: ProviderError,
    },
}
```

Keep crate-private `unknown_provider` and `provider_error` helpers only if they
make resolver construction clearer; they must construct these variants without
parallel storage. Implement `Display` by matching the variants. Implement
`Error::source()` as `None` for `UnknownProvider` and `Some(error)` for
`ProviderError`.

- [ ] **Step 3: Replace `ResolutionError` storage with its public enum**

Implement in `src/error/resolution_error.rs`:

```rust
#[derive(Clone, Debug)]
pub enum ResolutionError {
    /// Raw selector input failed normalization or syntax validation.
    InvalidSelector {
        /// Verbatim selector supplied by the caller.
        input: Box<str>,
        /// Zero-based chain position, or `None` for direct named selection.
        selector_index: Option<usize>,
        /// Parser error explaining why `input` was rejected.
        source: ProviderSelectorError,
    },
    /// A raw chained selection contained no selectors.
    EmptySelection,
    /// A valid normalized selector matched no registry entry.
    UnknownProvider {
        /// Normalized selector that matched no provider.
        selector: ProviderSelector,
    },
    /// Automatic selection was requested from an empty registry.
    EmptyRegistry,
    /// Every considered candidate failed or resolution stopped by policy.
    NoProviderSucceeded {
        /// Attempt diagnostics in encounter order.
        attempts: Box<[AttemptFailure]>,
    },
}
```

Implement custom `Display` with the current ordered messages. Implement
`Error::source()` as `Some(source)` only for `InvalidSelector`. Internal
construction may use crate-private helpers, but remove all `kind()`,
`selector_input()`, `selector_index()`, `selector_error()`,
`requested_selector()`, and `attempts()` methods.

- [ ] **Step 4: Move the remaining error files and update explicit imports**

Move each remaining public error type and kind file under `src/error/`. Move
the five remaining private representation files under `src/error/internal/`.
Update their imports to explicit paths such as:

```rust
use crate::error::internal::ProviderSelectorErrorRepr;
use crate::error::ProviderSelectorErrorKind;
```

Update non-error source files to imports such as:

```rust
use crate::error::{
    AttemptFailure,
    ProviderErrorKind,
    ResolutionError,
};
```

Keep registry-only internal imports under `crate::internal`.

- [ ] **Step 5: Make `lib.rs` expose only the error module path**

Add:

```rust
pub mod error;
```

Remove root module declarations and root `pub use` statements for every moved
error type. Remove declarations and re-exports for `AttemptFailureKind` and
`ResolutionErrorKind`. Update crate-level examples to import
`qubit_spi::error::ProviderError`.

- [ ] **Step 6: Run focused tests and verify GREEN**

Run:

```bash
cargo test --test integration_tests attempt_failure
cargo test --test integration_tests resolution_error
cargo test --test integration_tests provider_resolver
```

Expected: all focused tests pass; no removed kind or accessor references remain
inside `rs-spi`.

### Task 3: Add infallible ID conversion and align `ProviderDescriptor`

**Files:**
- Modify: `tests/provider_selector_tests.rs`
- Modify: `src/provider_selector.rs`
- Modify: `src/provider_registry_builder.rs`
- Modify: `src/provider_descriptor.rs`

**Interfaces:**
- Consumes: `ProviderId::as_str()` and existing `ProviderSelector` storage.
- Produces: `impl From<&ProviderId> for ProviderSelector`.

- [ ] **Step 1: Write the failing conversion test**

Add to `tests/provider_selector_tests.rs`:

```rust
/// Verifies that a validated canonical provider ID converts without reparsing.
#[test]
fn test_provider_selector_from_provider_id() {
    let id = ProviderId::new("file-command")
        .expect("test provider ID should be valid");

    let selector = ProviderSelector::from(&id);

    assert_eq!("file-command", selector.as_str());
}
```

- [ ] **Step 2: Run the focused test and verify RED**

Run:

```bash
cargo test --test integration_tests test_provider_selector_from_provider_id
```

Expected: compilation fails because `From<&ProviderId>` is not implemented.

- [ ] **Step 3: Implement the infallible conversion**

Add in `src/provider_selector.rs`, with complete Rustdoc on the trait method:

```rust
impl From<&ProviderId> for ProviderSelector {
    #[inline]
    fn from(id: &ProviderId) -> Self {
        Self(id.as_str().into())
    }
}
```

Import `ProviderId` explicitly. Replace both canonical reparses with:

```rust
let canonical_selector = ProviderSelector::from(descriptor.id());
```

and:

```rust
let canonical_selector = ProviderSelector::from(id);
```

Remove the obsolete impossible-panic Rustdoc from both call sites.

- [ ] **Step 4: Reorder and reclassify `ProviderDescriptor` methods**

Order the methods as `new`, `id`, `aliases`, `with_aliases`, `priority`,
`with_priority`. Remove `#[inline]` from iterative `with_aliases`. Change
`with_priority` to `#[inline(always)]`. Move each method together with its full
Rustdoc and attributes; do not change behavior.

- [ ] **Step 5: Run focused tests and verify GREEN**

Run:

```bash
cargo test --test integration_tests provider_selector
cargo test --test integration_tests provider_descriptor
cargo test --test integration_tests provider_registry_builder
```

Expected: all focused tests pass and production source contains no
`expect("canonical provider IDs are valid selectors")`.

### Task 4: Document reusable `ProviderSelection`

**Files:**
- Modify: `src/provider_selection.rs`
- Modify: `README.md`
- Modify: `README.zh_CN.md`

**Interfaces:**
- Consumes: `ProviderSelection::named` and `ProviderResolver::create`.
- Produces: a runnable Rustdoc example demonstrating one parsed selection reused for multiple creations.

- [ ] **Step 1: Add the complete type-level Rustdoc example**

Add a `# Examples` section to `ProviderSelection` using a minimal `ServiceSpec`
with `Config = ()` and `Output = &'static str`. Register one provider, construct
the resolver, parse `ProviderSelection::named("english")` once, and call:

```rust
let selection = ProviderSelection::named("english")?;
let first = resolver.create(&selection, &())?;
let second = resolver.create(&selection, &())?;

assert_eq!("hello", *first.service());
assert_eq!("hello", *second.service());
```

Import `ProviderError` from `qubit_spi::error` in the example. Use a hidden
`main` returning `Result` so the example is a runnable doctest.

- [ ] **Step 2: Update README error imports and selection explanation**

Update both READMEs so examples import `ProviderError` from
`qubit_spi::error`. Add one sentence explaining that `ProviderSelection` is
useful when validated configuration is reused across calls, while raw resolver
methods are convenient at runtime boundaries.

- [ ] **Step 3: Run doctests**

Run:

```bash
cargo test --doc
```

Expected: crate-level and `ProviderSelection` examples compile and pass.

### Task 5: Migrate `qubit-fs` to structured error matching

**Files:**
- Modify: `../rs-fs/src/provider/file_system_provider.rs`
- Modify: `../rs-fs/src/provider/file_system_registry_builder.rs`
- Modify: `../rs-fs/src/provider/file_system_registry.rs`
- Modify: affected `../rs-fs/tests/**/*.rs` imports

**Interfaces:**
- Consumes: `qubit_spi::error::{AttemptFailure, ProviderError, ProviderErrorKind, RegistrationError, ResolutionError}`.
- Produces: filesystem-domain error mapping without kind/accessor coordination.

- [ ] **Step 1: Run the downstream test before migration and verify RED**

After Task 2 changes the dependency API, run from `rs-fs`:

```bash
cargo test --test integration_tests file_system_registry
```

Expected: compilation fails on removed root error imports and removed
`ResolutionErrorKind` / accessors.

- [ ] **Step 2: Update imports to the single error namespace**

Use:

```rust
use qubit_spi::error::{
    AttemptFailure,
    ProviderErrorKind,
    RegistrationError,
    ResolutionError,
};
```

Import `ProviderError` through `qubit_spi::error` in provider traits and test
fixtures. Keep non-error registry, descriptor, ID, resolver, provider, and spec
types at the crate root.

- [ ] **Step 3: Replace filesystem resolution mapping with direct matching**

Implement the mapper shape:

```rust
fn map_resolution_error(error: ResolutionError) -> FsError {
    let kind = match &error {
        ResolutionError::UnknownProvider { .. } => {
            FsErrorKind::ProviderUnavailable
        }
        ResolutionError::NoProviderSucceeded { attempts }
            if attempts.iter().all(|attempt| {
                matches!(
                    attempt,
                    AttemptFailure::ProviderError { error, .. }
                        if matches!(
                            error.kind(),
                            ProviderErrorKind::Unsupported
                                | ProviderErrorKind::Unavailable
                        )
                )
            }) => FsErrorKind::ProviderUnavailable,
        ResolutionError::InvalidSelector { .. }
        | ResolutionError::EmptySelection
        | ResolutionError::EmptyRegistry
        | ResolutionError::NoProviderSucceeded { .. } => FsErrorKind::Other,
    };
    let message = error.to_string();
    FsError::with_source(kind, FsOperation::Provider, &message, error)
}
```

- [ ] **Step 4: Run focused downstream tests and verify GREEN**

Run:

```bash
cargo test --test integration_tests file_system_registry
cargo test --test integration_tests file_resource
```

Expected: filesystem provider registration and resolution tests pass.

### Task 6: Migrate `qubit-mime` and `qubit-magika`

**Files:**
- Modify: `../rs-mime/src/classifier/*provider*.rs`
- Modify: `../rs-mime/src/classifier/*registry*.rs`
- Modify: `../rs-mime/src/detector/*provider*.rs`
- Modify: `../rs-mime/src/detector/*registry*.rs`
- Modify: affected `../rs-mime/tests/**/*.rs` imports
- Modify: `../rs-magika/src/magika_mime_detector_provider.rs`
- Modify: affected `../rs-magika/tests/**/*.rs` imports

**Interfaces:**
- Consumes: the single `qubit_spi::error` namespace and direct enum variants.
- Produces: exact MIME domain mappings without placeholder selector names.

- [ ] **Step 1: Run MIME focused tests and verify RED**

Run from `rs-mime`:

```bash
cargo test --test integration_tests mime_detector_registry
cargo test --test integration_tests media_stream_classifier_registry
```

Expected: compilation fails on removed root error imports and kind/accessor APIs.

- [ ] **Step 2: Update MIME imports**

Import error types from:

```rust
use qubit_spi::error::{
    AttemptFailure,
    ProviderError,
    ProviderErrorKind,
    ProviderSelectorErrorKind,
    RegistrationError,
    ResolutionError,
};
```

Use only the subset required in each concrete file. Keep all non-error types at
the crate root.

- [ ] **Step 3: Rewrite detector mapping as exhaustive variant matching**

Match `ResolutionError` by value. For `InvalidSelector`, derive empty versus
invalid from `source.kind()` and use owned `input` directly. For
`UnknownProvider`, use `selector.as_str()`. For a singleton aggregate, match:

```rust
match attempts.as_ref() {
    [AttemptFailure::UnknownProvider { requested_selector }] => {
        MimeError::UnknownDetector {
            name: requested_selector.as_str().to_owned(),
        }
    }
    [AttemptFailure::ProviderError {
        provider_id,
        error,
        ..
    }] => match error.kind() {
        ProviderErrorKind::Unsupported | ProviderErrorKind::Unavailable => {
            MimeError::DetectorUnavailable {
                name: provider_id.as_str().to_owned(),
                reason: error.reason().to_owned(),
            }
        }
        ProviderErrorKind::InvalidConfiguration
        | ProviderErrorKind::InitializationFailed => MimeError::DetectorBackend {
            backend: provider_id.as_str().to_owned(),
            reason: error.reason().to_owned(),
        },
    },
    _ => MimeError::NoAvailableDetector {
        reason: ResolutionError::NoProviderSucceeded { attempts }.to_string(),
    },
}
```

Handle `EmptySelection` and `EmptyRegistry` with `NoAvailableDetector` using
their original `ResolutionError` display text. Do not introduce placeholder
names.

- [ ] **Step 4: Rewrite classifier mapping with the same concrete variants**

Use classifier domain variants (`EmptyClassifierName`,
`InvalidClassifierName`, `UnknownClassifier`, `ClassifierUnavailable`,
`ClassifierBackend`, and `NoAvailableClassifier`) with the same direct data
flow. The classifier does not need a singleton unknown-chain special case unless
its tests exercise a chain.

- [ ] **Step 5: Run MIME focused tests and verify GREEN**

Run:

```bash
cargo test --test integration_tests mime_detector_registry
cargo test --test integration_tests media_stream_classifier_registry
cargo test --test integration_tests mime_detector_provider
```

Expected: all focused MIME tests pass, including the singleton unknown-chain
mapping.

- [ ] **Step 6: Update Magika imports and verify**

Import `ProviderError` from `qubit_spi::error` in production and tests. Run from
`rs-magika`:

```bash
cargo test --test integration_tests magika_mime_detector_provider
```

Expected: Magika provider registration and source-preserving error tests pass.

### Task 7: Audit API removal and run repository verification

**Files:**
- Inspect: all changed files in `rs-spi`, `rs-fs`, `rs-mime`, and `rs-magika`
- Modify only: in-scope issues reported by prescribed checks

**Interfaces:**
- Consumes: all outputs from Tasks 1-6.
- Produces: aligned, CI-checked direct workspace migration with recorded evidence.

- [ ] **Step 1: Search for obsolete APIs and paths**

Run from the workspace root:

```bash
rg -n 'AttemptFailureKind|ResolutionErrorKind|\.kind\(\).*ResolutionError|selector_input\(|selector_error\(|requested_selector\(|attempts\(' rs-spi rs-fs rs-mime rs-magika --glob '*.rs'
rg -n 'qubit_spi::\{[^}]*ProviderError|qubit_spi::(Provider|Registration|Resolution).*Error' rs-spi rs-fs rs-mime rs-magika --glob '*.rs'
```

Expected: no obsolete resolution/attempt kinds, removed accessors, or root error
imports. Any matches for remaining validation-error methods must be inspected
rather than mechanically removed.

- [ ] **Step 2: Verify source organization**

Run:

```bash
rg --files rs-spi/src/error | sort
rg --files rs-spi/src/internal | sort
```

Expected: all error files are below `src/error/`; root `src/internal/` contains
only builder, registry, and provider-selection internals; the four obsolete
files are absent.

- [ ] **Step 3: Preserve pre-existing downstream changes**

Inspect, without reverting:

```bash
git --no-pager diff -- Cargo.toml Cargo.lock
```

Run separately in `rs-mime` and `rs-magika`. Confirm unrelated user edits remain
present and no source migration accidentally rewrote them.

- [ ] **Step 4: Run rs-spi alignment and CI**

From `rs-spi`:

```bash
./align-ci.sh
./ci-check.sh
```

Expected: both exit 0. If CI reports coverage below threshold, run exactly:

```bash
./coverage.sh json
```

Add only meaningful tests for reported in-scope uncovered branches, then rerun
alignment and CI.

- [ ] **Step 5: Run rs-fs alignment and CI**

From `rs-fs`, run `./align-ci.sh` then `./ci-check.sh`; run
`./coverage.sh json` only if CI reports coverage below threshold. Expected: all
applicable commands exit 0.

- [ ] **Step 6: Run rs-mime alignment and CI**

From `rs-mime`, run `./align-ci.sh` then `./ci-check.sh`; run
`./coverage.sh json` only if CI reports coverage below threshold. Reinspect
`Cargo.toml` and `Cargo.lock` afterward to separate user changes from script
effects. Expected: all applicable commands exit 0.

- [ ] **Step 7: Run rs-magika alignment and CI**

From `rs-magika`, run `./align-ci.sh` then `./ci-check.sh`; run
`./coverage.sh json` only if CI reports coverage below threshold. Reinspect
`Cargo.lock` afterward. Expected: all applicable commands exit 0.

- [ ] **Step 8: Review final diffs without committing**

Run separately in each repository:

```bash
git status --short
git --no-pager diff --check
git --no-pager diff
```

Expected: no whitespace errors, no unrelated source changes, no accidental
compatibility re-exports, and no Git commits or staged changes created by this
task.
