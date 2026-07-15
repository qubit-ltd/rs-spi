# rs-spi Error Boundary Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the overloaded SPI error model with lifecycle-specific errors, make selections invariant-safe, add raw resolver entry points and complete diagnostics, and migrate every current downstream.

**Architecture:** Validation errors are split at the ID, selector, descriptor, registration, selection, and resolution boundaries. `ProviderSelection` becomes opaque so every stored selection is valid; `ProviderResolver` accepts both validated selections and raw configuration input. Existing immutable registry and fallback semantics remain unchanged.

**Tech Stack:** Rust 2024, Rust 1.94, `thiserror` 2, Cargo integration tests, rustdoc, Clippy.

## Global Constraints

- Backward compatibility is not required; add no deprecated aliases or compatibility shims.
- Keep provider assembly explicit and registries immutable after build.
- Do not add global discovery, async provider APIs, availability preflight, or logging.
- Put all Rust tests under `tests/`; do not add inline test modules.
- Every production behavior change starts with a focused failing test.
- Preserve unrelated changes and handle `rs-spi`, `rs-fs`, `rs-mime`, and `rs-magika` as separate repositories.
- Do not commit, stage, or push unless the user explicitly authorizes it.

---

### Task 1: Split provider ID and selector validation errors

**Files:**
- Create: `src/provider_id_error.rs`
- Create: `src/provider_selector_error.rs`
- Modify: `src/provider_id.rs`
- Modify: `src/provider_selector.rs`
- Modify: `src/lib.rs`
- Modify: `tests/provider_id_tests.rs`
- Modify: `tests/provider_selector_tests.rs`

**Interfaces:**
- Produces: `ProviderIdError`, `ProviderIdErrorKind::{Empty, NonCanonical}`.
- Produces: `ProviderSelectorError`, `ProviderSelectorErrorKind::{Empty, Invalid}`.
- Changes: `ProviderId::new` and `FromStr` return `ProviderIdError`.
- Changes: `ProviderSelector::parse` and `FromStr` return `ProviderSelectorError`.

- [ ] **Step 1: Write failing ID validation tests**

Extend `tests/provider_id_tests.rs` to assert the wished-for error API:

```rust
use qubit_spi::{ProviderId, ProviderIdErrorKind};

#[test]
fn provider_id_reports_empty_and_noncanonical_input() {
    let empty = ProviderId::new("").expect_err("empty ID should fail");
    assert_eq!(ProviderIdErrorKind::Empty, empty.kind());
    assert_eq!(Some(""), empty.input());

    for input in ["File", " file", "file-", "-file", "文件"] {
        let error = ProviderId::new(input).expect_err("noncanonical ID should fail");
        assert_eq!(ProviderIdErrorKind::NonCanonical, error.kind());
        assert_eq!(Some(input), error.input());
    }
}
```

- [ ] **Step 2: Run the ID test and confirm RED**

Run: `cargo test --test provider_id_tests provider_id_reports_empty_and_noncanonical_input`

Expected: compilation fails because `ProviderIdErrorKind` and the new accessors do not exist.

- [ ] **Step 3: Implement ID-specific validation errors**

Create an opaque `ProviderIdError` backed by a private `thiserror::Error` enum, export it from `lib.rs`, and change canonical-token validation used by `ProviderId` to return it. The public surface is:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderIdErrorKind {
    Empty,
    NonCanonical,
}

impl ProviderIdError {
    pub const fn kind(&self) -> ProviderIdErrorKind;
    pub fn input(&self) -> Option<&str>;
}
```

Move shared syntax checking into a private boolean/helper that can be reused by selector validation without sharing error types.

- [ ] **Step 4: Verify ID tests GREEN**

Run: `cargo test --test provider_id_tests`

Expected: every provider ID test passes.

- [ ] **Step 5: Write failing selector diagnostic tests**

Extend `tests/provider_selector_tests.rs`:

```rust
use qubit_spi::{ProviderSelector, ProviderSelectorErrorKind};

#[test]
fn selector_errors_preserve_raw_and_normalized_input() {
    let empty = ProviderSelector::parse("  ").expect_err("blank selector should fail");
    assert_eq!(ProviderSelectorErrorKind::Empty, empty.kind());
    assert_eq!("  ", empty.input());
    assert_eq!(None, empty.normalized());

    let invalid = ProviderSelector::parse(" Bad Selector ")
        .expect_err("selector containing a space should fail");
    assert_eq!(ProviderSelectorErrorKind::Invalid, invalid.kind());
    assert_eq!(" Bad Selector ", invalid.input());
    assert_eq!(Some("bad selector"), invalid.normalized());
}
```

- [ ] **Step 6: Run the selector test and confirm RED**

Run: `cargo test --test provider_selector_tests selector_errors_preserve_raw_and_normalized_input`

Expected: compilation fails because `ProviderSelectorErrorKind` does not exist.

- [ ] **Step 7: Implement selector-specific errors**

Create `ProviderSelectorError` with a private `thiserror` representation, change `ProviderSelector::parse`/`FromStr`, and export:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectorErrorKind {
    Empty,
    Invalid,
}

impl ProviderSelectorError {
    pub const fn kind(&self) -> ProviderSelectorErrorKind;
    pub fn input(&self) -> &str;
    pub fn normalized(&self) -> Option<&str>;
}
```

- [ ] **Step 8: Verify the focused validation suite**

Run: `cargo test --test provider_id_tests --test provider_selector_tests`

Expected: all focused tests pass.

---

### Task 2: Split descriptor construction from registry conflicts

**Files:**
- Create: `src/provider_descriptor_error.rs`
- Modify: `src/provider_descriptor.rs`
- Modify: `src/registration_error.rs`
- Modify: `src/provider_registry_builder.rs`
- Modify: `src/lib.rs`
- Modify: `tests/provider_descriptor_tests.rs`
- Modify: `tests/provider_registry_builder_tests.rs`
- Modify: `tests/registration_error_tests.rs`

**Interfaces:**
- Consumes: `ProviderSelectorError` from Task 1.
- Produces: `ProviderDescriptorError`, `ProviderDescriptorErrorKind::{InvalidAlias, DuplicateAlias, AliasMatchesId}`.
- Narrows: `RegistrationError` to cross-entry `DuplicateSelector` only.

- [ ] **Step 1: Write failing descriptor error tests**

Add separate tests for invalid alias position/source, repeated normalized aliases, and alias matching ID:

```rust
#[test]
fn descriptor_reports_invalid_alias_with_position_and_source() {
    let error = ProviderDescriptor::new(ProviderId::new("file-command").expect("valid ID"))
        .with_aliases(["file", "bad alias"])
        .expect_err("invalid alias should fail");

    assert_eq!(ProviderDescriptorErrorKind::InvalidAlias, error.kind());
    assert_eq!(Some(1), error.alias_index());
    assert_eq!(Some("bad alias"), error.alias());
    assert!(Error::source(&error).is_some());
}
```

Add corresponding assertions for `DuplicateAlias` and `AliasMatchesId`, including the normalized alias.

- [ ] **Step 2: Run descriptor tests and confirm RED**

Run: `cargo test --test provider_descriptor_tests`

Expected: compilation fails because the descriptor-specific error API is missing.

- [ ] **Step 3: Implement descriptor-specific errors and alias validation**

Create the opaque error and export:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderDescriptorErrorKind {
    InvalidAlias,
    DuplicateAlias,
    AliasMatchesId,
}

impl ProviderDescriptorError {
    pub const fn kind(&self) -> ProviderDescriptorErrorKind;
    pub const fn alias_index(&self) -> Option<usize>;
    pub fn alias(&self) -> Option<&str>;
}
```

Use `#[source] ProviderSelectorError` for invalid aliases. Track the canonical selector separately from the alias `HashSet` so `AliasMatchesId` and `DuplicateAlias` remain distinguishable.

- [ ] **Step 4: Verify descriptor tests GREEN**

Run: `cargo test --test provider_descriptor_tests`

Expected: all descriptor tests pass.

- [ ] **Step 5: Narrow registration errors and lock conflict semantics**

Delete `EmptyIdentifier` and `InvalidIdentifier` from `RegistrationErrorKind`
and remove their constructors/accessors after descriptor construction no longer
uses them. Update `tests/registration_error_tests.rs` to characterize only
duplicate registry ownership. Update builder tests to assert that conflicts
retain selector, existing owner, and new owner, and that a rejected provider
reserves no selectors.

Run: `cargo test --test provider_descriptor_tests --test provider_registry_builder_tests --test registration_error_tests`

Expected: tests pass after adapting imports to the narrowed `RegistrationErrorKind`.

---

### Task 3: Make provider selections opaque and invariant-safe

**Files:**
- Create: `src/provider_selection_error.rs`
- Modify: `src/provider_selection.rs`
- Modify: `src/lib.rs`
- Modify: `tests/provider_selection_tests.rs`
- Modify: `tests/provider_resolver_tests.rs`

**Interfaces:**
- Consumes: `ProviderSelectorError`.
- Produces: opaque `ProviderSelection`, `ProviderSelectionKind`, and `ProviderSelectionError`.
- Produces constructors `auto`, `named`, `chain` and accessors `kind`, `selector`, `selectors`.

- [ ] **Step 1: Rewrite selection tests against the opaque API**

Replace public-variant matching with accessor assertions and add empty/invalid tests:

```rust
#[test]
fn selection_construction_enforces_invariants() {
    let automatic = ProviderSelection::auto();
    assert_eq!(ProviderSelectionKind::Auto, automatic.kind());

    let named = ProviderSelection::named(" File+Command ").expect("valid selector");
    assert_eq!(ProviderSelectionKind::Named, named.kind());
    assert_eq!(Some("file+command"), named.selector().map(ProviderSelector::as_str));

    let empty = ProviderSelection::chain(Vec::<&str>::new())
        .expect_err("empty chain should fail");
    assert_eq!(ProviderSelectionErrorKind::EmptyChain, empty.kind());

    let invalid = ProviderSelection::chain(["valid", "bad selector"])
        .expect_err("invalid chain selector should fail");
    assert_eq!(ProviderSelectionErrorKind::InvalidSelector, invalid.kind());
    assert_eq!(Some(1), invalid.selector_index());
    assert_eq!(Some("bad selector"), invalid.selector_input());
}
```

- [ ] **Step 2: Run selection tests and confirm RED**

Run: `cargo test --test provider_selection_tests`

Expected: compilation fails because the new kinds/accessors are absent and public variants are no longer the desired API.

- [ ] **Step 3: Implement the opaque selection and its error**

Use a private representation enum and expose:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectionKind { Auto, Named, Chain }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectionErrorKind { InvalidSelector, EmptyChain }
```

The selection error retains index, raw selector, and a `ProviderSelectorError` source. Provide crate-private access to the representation for `ProviderResolver`; do not expose construction bypasses.

- [ ] **Step 4: Adapt resolver tests to constructors and verify GREEN**

Replace `ProviderSelection::Auto` with `ProviderSelection::auto()` and public enum matches with accessors.

Run: `cargo test --test provider_selection_tests --test provider_resolver_tests`

Expected: selection tests pass; resolver tests compile against the new construction API while behavior remains unchanged.

---

### Task 4: Redesign resolution errors and raw resolver entry points

**Files:**
- Modify: `src/resolution_error.rs`
- Modify: `src/provider_resolver.rs`
- Modify: `src/provider_registry.rs`
- Modify: `tests/resolution_error_tests.rs`
- Modify: `tests/provider_registry_tests.rs`
- Modify: `tests/provider_resolver_tests.rs`

**Interfaces:**
- Consumes: opaque `ProviderSelection` and `ProviderSelectorError`.
- Produces: `ResolutionErrorKind::{InvalidSelector, EmptySelection, UnknownProvider, EmptyRegistry, NoProviderSucceeded}`.
- Produces: public `ProviderResolver::{create_auto, create_named, create_chain}`.
- Produces: `Display + Error` for `AttemptFailure` and detailed aggregate display.

- [ ] **Step 1: Write failing raw resolver API tests**

Add focused tests that call the wished-for APIs:

```rust
#[test]
fn raw_named_resolution_preserves_invalid_selector_input() {
    let resolver = ProviderResolver::<GreetingSpec>::new(
        ProviderRegistry::default(),
        FallbackPolicy::OnAbsence,
    );
    let error = resolver
        .create_named(" Bad Selector ", &())
        .expect_err("invalid raw selector should fail");

    assert_eq!(ResolutionErrorKind::InvalidSelector, error.kind());
    assert_eq!(Some(" Bad Selector "), error.selector_input());
    assert_eq!(None, error.selector_index());
    assert!(Error::source(&error).is_some());
}

#[test]
fn raw_chain_reports_invalid_selector_position_and_empty_input() {
    let resolver = ProviderResolver::<GreetingSpec>::new(
        ProviderRegistry::default(),
        FallbackPolicy::OnAbsence,
    );
    let invalid = resolver
        .create_chain(["valid", "bad selector"], &())
        .expect_err("invalid raw chain selector should fail");
    assert_eq!(ResolutionErrorKind::InvalidSelector, invalid.kind());
    assert_eq!(Some(1), invalid.selector_index());
    assert_eq!(Some("bad selector"), invalid.selector_input());

    let empty = resolver
        .create_chain(Vec::<&str>::new(), &())
        .expect_err("empty raw chain should fail");
    assert_eq!(ResolutionErrorKind::EmptySelection, empty.kind());
}
```

- [ ] **Step 2: Run the new raw resolver tests and confirm RED**

Run: `cargo test --test provider_resolver_tests raw_`

Expected: compilation fails because the raw resolver methods and new accessors do not exist.

- [ ] **Step 3: Write failing empty-registry and aggregate-display tests**

Add tests asserting:

```rust
let error = resolver.create_auto(&()).expect_err("empty registry should fail");
assert_eq!(ResolutionErrorKind::EmptyRegistry, error.kind());
assert!(error.attempts().is_empty());
```

For two failed providers, assert `error.to_string()` contains both canonical IDs and both reasons in encounter order. Assert `AttemptFailure::to_string()` includes its selector/provider context and reason.

- [ ] **Step 4: Run the diagnostic tests and confirm RED**

Run: `cargo test --test provider_resolver_tests --test resolution_error_tests`

Expected: new classification/display assertions fail against the count-only aggregate error.

- [ ] **Step 5: Implement the new resolution representation**

Replace the private representation with variants carrying:

```rust
InvalidSelector {
    input: Box<str>,
    selector_index: Option<usize>,
    source: ProviderSelectorError,
}
EmptySelection,
UnknownProvider { selector: ProviderSelector },
EmptyRegistry,
NoProviderSucceeded { attempts: Box<[AttemptFailure]> },
```

Add `selector_index() -> Option<usize>`. Keep `attempts()` empty for non-aggregate kinds. Implement `Display` manually where necessary to render ordered attempt summaries.

- [ ] **Step 6: Implement raw resolver methods and consolidate internal paths**

Expose `create_auto`, `create_named`, and generic `create_chain`. Parse raw inputs exactly once, map selector errors into `ResolutionError::InvalidSelector`, and report empty raw chains before attempting resolution. Rename private helpers so public names remain unambiguous, for example `create_named_selector` and `create_selector_chain`.

In automatic resolution, return `EmptyRegistry` before allocating an attempt vector. Resolve each automatic index once per loop iteration rather than calling `resolved_at(index)` repeatedly.

- [ ] **Step 7: Verify resolution and resolver tests GREEN**

Run: `cargo test --test provider_registry_tests --test provider_resolver_tests --test resolution_error_tests`

Expected: all focused tests pass with the new API and diagnostics.

---

### Task 5: Complete registry and created-service APIs

**Files:**
- Modify: `src/created_service.rs`
- Modify: `src/provider_registry.rs`
- Modify: `tests/created_service_tests.rs`
- Modify: `tests/provider_registry_tests.rs`

**Interfaces:**
- Produces: `CreatedService::into_parts(self) -> (ProviderId, T)`.
- Produces: `ProviderRegistry::len(&self) -> usize`.

- [ ] **Step 1: Write failing ownership and length tests**

```rust
#[test]
fn created_service_decomposes_into_owned_parts() {
    let created = CreatedService::new(ProviderId::new("memory").expect("valid ID"), 42_u8);
    let (provider_id, service) = created.into_parts();
    assert_eq!("memory", provider_id.as_str());
    assert_eq!(42, service);
}
```

Add registry assertions for zero and nonzero `len()`, and consistency with `is_empty()`.

- [ ] **Step 2: Run focused tests and confirm RED**

Run: `cargo test --test created_service_tests --test provider_registry_tests`

Expected: compilation fails because `into_parts` and `len` do not exist.

- [ ] **Step 3: Implement the two minimal APIs**

Move both private fields out in `into_parts`; return `self.inner.entries.len()` from `len`; implement `is_empty` through the slice or `len() == 0` consistently.

- [ ] **Step 4: Verify focused tests GREEN**

Run: `cargo test --test created_service_tests --test provider_registry_tests`

Expected: all focused tests pass.

---

### Task 6: Migrate rs-fs to the precise error boundaries

**Files:**
- Modify: `../rs-fs/src/provider/file_system_registry.rs`
- Modify: `../rs-fs/tests/provider/file_system_registry_tests.rs`

**Interfaces:**
- Consumes: `ProviderResolver::create_named`, `ProviderIdError`, `ProviderDescriptorError`, narrowed `RegistrationError`, redesigned `ResolutionError`.
- Preserves: existing `FsErrorKind` behavior for invalid URI/provider input and provider failures.

- [ ] **Step 1: Lock filesystem runtime-selector behavior before migration**

Extend filesystem registry tests to assert an unknown valid URI scheme maps to
`ProviderUnavailable` with a non-empty diagnostic and a provider creation
failure still maps to `FsErrorKind::Other`. These assertions protect the domain
behavior while the upstream API fails to compile during migration.

- [ ] **Step 2: Run rs-fs tests and confirm RED after the SPI API changes**

Run: `cargo test --test lib_tests provider::file_system_registry_tests`

Expected: compilation fails at obsolete `RegistrationErrorKind` variants and `ProviderSelection::named` mapping.

- [ ] **Step 3: Migrate filesystem selection and mappings**

Change `FileSystemRegistry::fs` to call:

```rust
self.resolver
    .create_named(uri.scheme.as_str(), &config)
    .map(CreatedService::into_service)
    .map_err(map_resolution_error)
```

Keep registration mapping focused on duplicate registry selectors. Add separate construction mappings only at call sites that can genuinely receive ID or descriptor errors.

- [ ] **Step 4: Verify rs-fs GREEN**

Run: `cargo test --all-targets --all-features`

Expected: all filesystem tests pass.

---

### Task 7: Migrate rs-mime and rs-magika

**Files:**
- Modify: `../rs-mime/src/detector/mime_detector_registry.rs`
- Modify: `../rs-mime/src/detector/mime_detector_registry_builder.rs`
- Modify: `../rs-mime/src/detector/repository_mime_detector_provider.rs`
- Modify: `../rs-mime/src/detector/file_command_mime_detector_provider.rs`
- Modify: `../rs-mime/src/classifier/media_stream_classifier_registry.rs`
- Modify: `../rs-mime/src/classifier/media_stream_classifier_registry_builder.rs`
- Modify: `../rs-mime/src/classifier/ffprobe_command_media_stream_classifier_provider.rs`
- Modify: `../rs-mime/tests/detector/mime_detector_registry_tests.rs`
- Modify: `../rs-mime/tests/classifier/media_stream_classifier_registry_tests.rs`
- Modify: `../rs-magika/src/magika_mime_detector_provider.rs`
- Modify: `../rs-magika/tests/magika_mime_detector_provider_tests.rs`
- Modify: `README.md`
- Modify: `README.zh_CN.md`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: every public error/API produced by Tasks 1-5.
- Preserves: MIME-domain result types and explicit Magika registration.

- [ ] **Step 1: Add failing MIME error-boundary tests**

Extend detector and classifier registry tests to assert:

- invalid explicit selectors map to the existing invalid-name domain errors;
- unknown valid selectors map to existing unknown-name errors;
- empty automatic registries return non-empty no-available diagnostics;
- exhausted chains include every provider reason through SPI aggregate display.

- [ ] **Step 2: Run focused MIME tests and confirm RED/compile failure**

Run: `cargo test --test mod detector::mime_detector_registry_tests classifier::media_stream_classifier_registry_tests`

Expected: compilation fails at old SPI error variants and public `ProviderSelection` variants.

- [ ] **Step 3: Migrate MIME detector and classifier registries**

Use `create_named`, `create_auto`, and `create_chain` directly where configuration input is consumed. Keep `ProviderSelection` only where a validated selection is intentionally retained. Split domain conversion helpers by their precise input error types; use `ResolutionError::to_string()` for aggregate diagnostics after verifying the new display contains all attempts.

- [ ] **Step 4: Verify rs-mime GREEN**

Run: `cargo test --all-targets --all-features`

Expected: all MIME tests pass.

- [ ] **Step 5: Add a failing Magika descriptor regression test**

Assert the descriptor factory still returns canonical ID `magika`, normalized aliases, and priority 20 after the descriptor error type changes.

Run: `cargo test --test magika_mime_detector_provider_tests`

Expected: compilation fails until the factory imports and result handling are migrated.

- [ ] **Step 6: Migrate Magika descriptor construction and verify GREEN**

Adapt imports and `expect` messages to `ProviderIdError` and `ProviderDescriptorError` without changing explicit registration or source-preserving provider creation.

Run: `cargo test --all-targets --all-features`

Expected: all Magika tests pass.

- [ ] **Step 7: Update SPI documentation**

Update both READMEs and crate rustdoc so examples use `ProviderSelection::auto()` or raw resolver methods, document lifecycle-specific errors, empty registry/selection behavior, detailed attempt diagnostics, `len()`, and `into_parts()`. Remove every reference to obsolete registration-error validation variants and public selection enum construction.

Run: `cargo test --doc`

Expected: the updated crate example compiles and passes.

---

### Task 8: Full verification and repository review

**Files:** all files changed by Tasks 1-7.

**Interfaces:**
- Consumes: completed implementations and downstream migrations.
- Produces: verified, reviewable working trees with no staged or committed changes.

- [ ] **Step 1: Run formatting checks in all four repositories**

Run `cargo fmt --all -- --check` in `rs-spi`, `rs-fs`, `rs-mime`, and `rs-magika`.

Expected: all commands exit 0 with no formatting diff.

- [ ] **Step 2: Run complete tests in all four repositories**

Run `cargo test --all-targets --all-features` in each repository, then run `cargo test --doc --all-features` in each repository.

Expected: every command exits 0 with zero failed tests.

- [ ] **Step 3: Run Clippy and documentation builds**

Run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo doc --no-deps --all-features` in each repository.

Expected: all commands exit 0 with no warning or rustdoc error.

- [ ] **Step 4: Search for stale API usage**

Run:

```bash
rg -n 'RegistrationErrorKind::(EmptyIdentifier|InvalidIdentifier)|ProviderSelection::(Auto|Named|Chain)' \
    ../rs-spi ../rs-fs ../rs-mime ../rs-magika \
    --glob '*.rs' --glob '*.md'
```

Expected: no stale source or documentation references.

- [ ] **Step 5: Review repository diffs and cleanliness boundaries**

In each repository run `git --no-pager diff --check`, `git status --short`, and `git --no-pager diff --stat`. Review the full diffs for unrelated edits and confirm no file is staged or committed.

Expected: only the approved redesign, tests, documentation, and downstream migrations are present.
