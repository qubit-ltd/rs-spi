# rs-spi Error API Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove redundant validation and registration error inspection APIs, migrate direct consumers to enum matching, update the filesystem provider guide to the 0.7 contract, and align one forwarding inline attribute.

**Architecture:** Keep each public error as one `#[non_exhaustive]` enum. Consumers match variant-specific data directly and retain only accessors whose value exists across every current variant; `ProviderErrorKind`, `ProviderSelectionKind`, and `ResolutionTermination` remain unchanged.

**Tech Stack:** Rust 1.94, Cargo, `thiserror`, Markdown documentation, repository-provided CI scripts.

## Global Constraints

- Breaking API changes are authorized; no compatibility aliases are added.
- Selector lookup allocation and performance optimization remain out of scope.
- Preserve validation, registration, resolution, fallback, and source-chain behavior.
- Keep all Rust tests in each crate's external `tests/` tree.
- Do not add dependencies.
- Do not run `git add`, `git commit`, or `git push` without separate authorization.
- Treat `rs-spi`, `rs-mime`, `rs-fs`, and `rs-magika` as independent repositories.

---

### Task 1: Collapse the rs-spi error API to one representation

**Files:**
- Modify: `rs-spi/src/error/mod.rs`
- Modify: `rs-spi/src/error/provider_descriptor_error.rs`
- Modify: `rs-spi/src/error/provider_id_error.rs`
- Modify: `rs-spi/src/error/provider_selection_error.rs`
- Modify: `rs-spi/src/error/provider_selector_error.rs`
- Modify: `rs-spi/src/error/registration_error.rs`
- Modify: `rs-spi/src/provider_registry.rs`
- Modify: `rs-spi/tests/error/mod.rs`
- Modify: `rs-spi/tests/provider_descriptor_tests.rs`
- Modify: `rs-spi/tests/provider_id_tests.rs`
- Modify: `rs-spi/tests/provider_selection_tests.rs`
- Modify: `rs-spi/tests/provider_selector_tests.rs`
- Modify: `rs-spi/tests/error/registration_error_tests.rs`
- Delete: `rs-spi/src/error/provider_descriptor_error_kind.rs`
- Delete: `rs-spi/src/error/provider_id_error_kind.rs`
- Delete: `rs-spi/src/error/provider_selection_error_kind.rs`
- Delete: `rs-spi/src/error/provider_selector_error_kind.rs`
- Delete: `rs-spi/src/error/registration_error_kind.rs`
- Delete: `rs-spi/tests/error/provider_descriptor_error_kind_tests.rs`
- Delete: `rs-spi/tests/error/provider_id_error_kind_tests.rs`
- Delete: `rs-spi/tests/error/provider_selection_error_kind_tests.rs`
- Delete: `rs-spi/tests/error/provider_selector_error_kind_tests.rs`
- Delete: `rs-spi/tests/error/registration_error_kind_tests.rs`

**Interfaces:**
- Consumes: existing public validation and registration error enums.
- Produces: the same enums without parallel Kind types, `kind()` methods, or variant-specific optional accessors; retains `ProviderDescriptorError::alias`, `ProviderIdError::input`, and `ProviderSelectorError::input`.

- [ ] **Step 1: Add the failing public migration doctest**

Append this migration contract to the module documentation in `src/error/mod.rs`:

```rust
//! # Removed parallel error classifications
//!
//! Validation and registration errors are matched directly. The former
//! parallel Kind types are intentionally unavailable:
//!
//! ```compile_fail
//! use qubit_spi::error::{
//!     ProviderDescriptorErrorKind,
//!     ProviderIdErrorKind,
//!     ProviderSelectionErrorKind,
//!     ProviderSelectorErrorKind,
//!     RegistrationErrorKind,
//! };
//! # fn main() {}
//! ```
```

- [ ] **Step 2: Run the doctest and verify RED**

Run from `rs-spi`:

```bash
cargo test --doc
```

Expected: FAIL because the `compile_fail` block still compiles while all five obsolete Kind types exist.

- [ ] **Step 3: Rewrite behavioral tests around direct enum matching**

Remove all Kind imports and `kind()` assertions. Replace optional-accessor assertions with direct matches that bind the correlated fields. For example:

```rust
let ProviderDescriptorError::InvalidAlias {
    alias_index,
    alias,
    source,
} = error
else {
    panic!("invalid alias should retain position, input, and source");
};
assert_eq!(1, alias_index);
assert_eq!("bad alias", alias.as_ref());
assert_eq!("bad alias", source.input());
```

Use the same pattern for:

```rust
ProviderIdError::Empty { input }
ProviderIdError::NonCanonical { input }
ProviderSelectorError::Empty { input }
ProviderSelectorError::Invalid { input, normalized }
ProviderSelectionError::EmptyChain
ProviderSelectionError::InvalidSelector {
    selector_index,
    selector_input,
    source,
}
RegistrationError::DuplicateSelector {
    selector,
    existing_provider,
    provider,
}
```

Keep assertions for `alias()`, `input()`, `Error::source`, and all existing diagnostic fields. Remove the five obsolete Kind test-module declarations from `tests/error/mod.rs` and delete their dedicated test files.

- [ ] **Step 4: Remove redundant production APIs**

In `src/error/mod.rs`, delete the five Kind module declarations and re-exports. Delete their source files.

From the five error implementations, remove Kind imports and `kind()` methods. Also remove exactly these accessors:

```text
ProviderDescriptorError::alias_index
ProviderDescriptorError::selector_error
ProviderSelectionError::selector_index
ProviderSelectionError::selector_input
ProviderSelectionError::selector_error
ProviderSelectionError::is_empty_chain
ProviderSelectorError::normalized
ProviderSelectorError::is_empty
RegistrationError::selector
RegistrationError::existing_provider
RegistrationError::provider
```

Retain these cross-variant queries unchanged:

```text
ProviderDescriptorError::alias
ProviderIdError::input
ProviderSelectorError::input
```

Change the pure forwarding attribute in `ProviderRegistry::builder()` to:

```rust
#[inline(always)]
#[must_use]
pub fn builder() -> ProviderRegistryBuilder<S> {
    ProviderRegistryBuilder::new()
}
```

- [ ] **Step 5: Verify GREEN and API absence**

Run from `rs-spi`:

```bash
cargo test --doc
cargo test --test integration_tests
```

Expected: both commands PASS.

Then run:

```bash
rg -n 'ProviderDescriptorErrorKind|ProviderIdErrorKind|ProviderSelectionErrorKind|ProviderSelectorErrorKind|RegistrationErrorKind|alias_index\(|selector_error\(|selector_index\(|selector_input\(|is_empty_chain\(|normalized\(|\.is_empty\(\)|existing_provider\(|\.provider\(' src tests --glob '*.rs'
```

Expected: no matches for the removed SPI APIs. If the broad method patterns find unrelated standard-library calls, narrow the search to the affected error files and confirm no removed declarations or call sites remain.

---

### Task 2: Migrate qubit-mime to direct error matching

**Files:**
- Modify: `rs-mime/src/detector/mime_detector_registry.rs`
- Modify: `rs-mime/src/classifier/media_stream_classifier_registry.rs`
- Modify: `rs-mime/src/mime_config.rs`
- Modify tests only if current assertions depend on the removed accessors after production migration.

**Interfaces:**
- Consumes: cleaned `qubit_spi::error::{RegistrationError, ProviderSelectionError, ProviderSelectorError}` enums from Task 1.
- Produces: detector and classifier domain errors without any removed SPI accessor usage.

- [ ] **Step 1: Compile the downstream before migration and verify RED**

Run from `rs-mime` after Task 1:

```bash
cargo test --test integration_tests
```

Expected: compilation FAILS at calls to `RegistrationError::selector`, `ProviderSelectorError::is_empty`, and `ProviderSelectionError::selector_input`.

- [ ] **Step 2: Migrate registration adapters**

Replace detector registration mapping with direct matching:

```rust
pub(crate) fn detector_registration_error(
    error: RegistrationError,
) -> MimeError {
    let reason = error.to_string();
    match error {
        RegistrationError::DuplicateSelector { selector, .. } => {
            MimeError::DuplicateDetectorName {
                name: selector.into(),
            }
        }
        _ => MimeError::NoAvailableDetector { reason },
    }
}
```

Apply the classifier equivalent, mapping the known variant to
`DuplicateClassifierName` and the wildcard to `NoAvailableClassifier`.

- [ ] **Step 3: Migrate selector-source classification**

Import `ProviderSelectorError` and replace both `source.is_empty()` calls with:

```rust
if matches!(source, ProviderSelectorError::Empty { .. }) {
```

Preserve all existing invalid-name, unknown-provider, unavailable, backend, and aggregate mappings.

- [ ] **Step 4: Migrate MIME configuration selection adapters**

Replace `selector_input()` branching with direct enum matching. The detector adapter becomes:

```rust
fn detector_selection_error(error: ProviderSelectionError) -> MimeError {
    let reason = error.to_string();
    match error {
        ProviderSelectionError::InvalidSelector {
            selector_input, ..
        } => MimeError::InvalidDetectorName {
            name: selector_input.into(),
            reason,
        },
        ProviderSelectionError::EmptyChain => MimeError::EmptyDetectorName,
        _ => MimeError::NoAvailableDetector { reason },
    }
}
```

Apply the classifier equivalent with `InvalidClassifierName`,
`EmptyClassifierName`, and `NoAvailableClassifier`.

- [ ] **Step 5: Verify GREEN and removed-use absence**

Run from `rs-mime`:

```bash
cargo test --test integration_tests
```

Expected: PASS.

Run:

```bash
rg -n 'selector_input\(|selector_error\(|is_empty_chain\(|source\.is_empty\(\)|error\.selector\(\)' src tests --glob '*.rs'
```

Expected: no matches referring to removed SPI accessors.

---

### Task 3: Rewrite the qubit-fs provider guide for SPI 0.7

**Files:**
- Modify: `rs-fs/doc/user_guide.md:1167`
- Modify: `rs-fs/doc/user_guide.zh_CN.md:1175`

**Interfaces:**
- Consumes: `FileSystemSpec::Output = Arc<dyn FileSystem>`, `FileSystemRegistry::builder`, `FileSystemRegistryBuilder::register`, and the `qubit-spi` 0.7 provider contract.
- Produces: matching English and Chinese provider-registration examples using immutable startup assembly.

- [ ] **Step 1: Replace the English provider example**

Use these imports and signatures:

```rust
use std::sync::Arc;

use qubit_fs::{
    FileSystem,
    FileSystemConfig,
    FileSystemRegistryBuilder,
    FileSystemSpec,
    FsResult,
};
use qubit_spi::error::ProviderError;
use qubit_spi::{
    ProviderDescriptor,
    ProviderId,
    ServiceProvider,
};

impl ServiceProvider<FileSystemSpec> for MemoryFileSystemProvider {
    fn create(
        &self,
        _config: &FileSystemConfig,
    ) -> Result<Arc<dyn FileSystem>, ProviderError> {
        Ok(Arc::new(MemoryFileSystem::default()))
    }
}

pub fn register_provider(
    builder: &mut FileSystemRegistryBuilder,
) -> FsResult<()> {
    let descriptor = ProviderDescriptor::new(
        ProviderId::new("memory")
            .expect("memory provider ID should be valid"),
    )
    .with_aliases(["mem"])
    .expect("memory provider aliases should be valid");
    builder.register(descriptor, MemoryFileSystemProvider)
}
```

Update application assembly to create `let mut builder = FileSystemRegistry::builder();`, register providers into it, then call `let registry = builder.build();` before resolving URIs.

- [ ] **Step 2: Apply the equivalent Chinese example**

Use exactly the same Rust API and code shape as the English guide. Translate only explanatory prose; keep identifiers and error messages consistent between both examples.

- [ ] **Step 3: Verify stale API removal**

Run from the workspace root:

```bash
rg -n 'ProviderCreateError|ProviderRegistryError|fn descriptor\(|fn create_box\(|registry\.register\(' rs-fs/doc/user_guide.md rs-fs/doc/user_guide.zh_CN.md
```

Expected: no matches in the updated provider-registration and application-assembly sections. Review any matches elsewhere in the guides and update them only when they describe the obsolete SPI contract.

---

### Task 4: Run repository-prescribed verification

**Files:**
- Inspect changes produced by alignment scripts in each repository before continuing.

**Interfaces:**
- Consumes: completed Tasks 1-3.
- Produces: fresh CI-equivalent evidence for `rs-spi`, `rs-mime`, `rs-fs`, and compatibility consumer `rs-magika`.

- [ ] **Step 1: Verify rs-spi**

Run from `rs-spi`:

```bash
./align-ci.sh
./ci-check.sh
```

Expected: both exit 0. Inspect the alignment diff before accepting it. If CI reports coverage below threshold, run exactly:

```bash
./coverage.sh json
```

Add tests only for business or error branches exposed by the report, then rerun alignment and CI.

- [ ] **Step 2: Verify rs-mime**

Run the same conditional sequence from `rs-mime`:

```bash
./align-ci.sh
./ci-check.sh
```

Expected: both exit 0. Run `./coverage.sh json` only if CI reports coverage below threshold.

- [ ] **Step 3: Verify rs-fs**

Run the same conditional sequence from `rs-fs`:

```bash
./align-ci.sh
./ci-check.sh
```

Expected: both exit 0. Run `./coverage.sh json` only if CI reports coverage below threshold.

- [ ] **Step 4: Verify rs-magika compatibility**

Run from `rs-magika`:

```bash
./align-ci.sh
./ci-check.sh
```

Expected: both exit 0 without production changes. Run `./coverage.sh json` only if CI reports coverage below threshold.

- [ ] **Step 5: Audit final scope**

For each repository separately, run:

```bash
git status --short
git --no-pager diff
```

Confirm that changes are limited to the files named in this plan plus formatting adjustments made by the repository scripts. Do not stage or commit them.
