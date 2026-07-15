# rs-spi Contract Hardening Implementation Plan

> Execute in small test-first increments. Preserve the existing uncommitted migrations in `rs-fs` and `rs-magika`; do not commit or stage changes.

**Goal:** Harden `rs-spi` contracts and align its directly affected MIME/Magika downstreams with the approved breaking API cleanup.

**Architecture:** Keep registry construction mutable and startup-only, registries immutable and shared, and resolvers as owning runtime facades. Remove the public registration transport type, use private builder entries, expose resolver state read-only, and implement opaque structured errors through private `thiserror` representations.

**Tech Stack:** Rust 2024, Cargo, `thiserror` 2, integration tests, rustdoc, Clippy.

---

### Task 1: Lock registration and resolver contracts with failing tests

**Files:**
- Modify: `tests/provider_registry_builder_tests.rs`
- Modify: `tests/provider_resolver_tests.rs`
- Modify: `tests/provider_registry_tests.rs`

1. Add tests for owned/shared registrations, atomic duplicate rejection, priority tie-breaking, and concurrent immutable access.
2. Add tests requiring `ProviderResolver::registry`, `fallback_policy`, `Clone`, and `Debug`.
3. Run the focused test targets and confirm the new resolver tests fail for missing APIs while existing registration behavior remains characterized.

### Task 2: Remove public ProviderRegistration and simplify builder internals

**Files:**
- Delete: `src/provider_registration.rs`
- Modify: `src/lib.rs`
- Modify: `src/provider_registry_builder.rs`

1. Replace `ProviderRegistration<S>` storage with a private builder entry.
2. Make `register` and `register_shared` converge on one private validation-and-insert function.
3. Remove the module and public re-export.
4. Run builder and registry tests.

### Task 3: Add resolver observation and basic traits

**Files:**
- Modify: `src/provider_resolver.rs`
- Modify: `tests/provider_resolver_tests.rs`

1. Add read-only registry and fallback-policy accessors.
2. Implement `Clone` and bounded `Debug` without provider `Debug` bounds.
3. Complete fallback matrices, chain alias de-duplication, and resolver concurrency tests.
4. Run resolver tests and confirm all pass.

### Task 4: Restructure errors with thiserror and improve diagnostics

**Files:**
- Modify: `src/provider_error.rs`
- Modify: `src/registration_error.rs`
- Modify: `src/resolution_error.rs`
- Modify: `src/provider_registry.rs`
- Modify: `tests/provider_error_tests.rs`
- Modify: `tests/provider_registry_builder_tests.rs`
- Modify: `tests/provider_registry_tests.rs`
- Modify: `tests/provider_resolver_tests.rs`

1. Add failing tests for invalid raw selectors, source traversal, duplicate-owner diagnostics, and explicit attempt kind.
2. Introduce private `thiserror::Error` representations behind opaque public wrappers.
3. Preserve raw selector input and its validation source in `ProviderRegistry::resolve`.
4. Add explicit attempt-failure classification and update resolver construction sites.
5. Run all `rs-spi` tests.

### Task 5: Align downstream resolver ownership and MIME exports/docs

**Files:**
- Modify: `../rs-fs/src/provider/file_system_registry.rs`
- Modify: `../rs-mime/src/detector/mime_detector_registry.rs`
- Modify: `../rs-mime/src/classifier/media_stream_classifier_registry.rs`
- Modify: `../rs-mime/src/lib.rs`
- Modify: `../rs-mime/src/detector/mime_detector_provider.rs`
- Modify other `rs-mime` rustdoc examples discovered by compiler/search as required.

1. Remove duplicated registry fields where they only mirror resolver ownership.
2. Remove all `qubit-spi` re-exports from `qubit-mime`.
3. Update `rs-mime` imports and rustdoc to the current explicit descriptor/provider/builder/resolver model.
4. Run `rs-fs` and `rs-mime` tests and documentation builds.

### Task 6: Restore and migrate Magika behavior tests

**Files:**
- Restore then modify: `../rs-magika/tests/detector/magika_mime_detector_tests.rs`
- Restore then modify: `../rs-magika/tests/magika_mime_detector_provider_tests.rs`
- Modify: `../rs-magika/src/magika_mime_detector_provider.rs`
- Modify imports/docs in `../rs-magika/src/lib.rs`, READMEs, and manifest only as required.

1. Run the explicitly authorized `git restore --source=HEAD --` for the two test files.
2. Run the restored tests to capture migration compile failures.
3. Migrate registrations to descriptor plus provider builder calls and import SPI traits/types directly from `qubit-spi`.
4. Change Magika initialization mapping to retain the backend error as `source`.
5. Add an assertion that traverses the initialization error source chain.
6. Run all Magika tests with relevant feature combinations.

### Task 7: Update rs-spi READMEs

**Files:**
- Modify: `README.md`
- Modify: `README.zh_CN.md`

1. Update examples and architecture descriptions to the final API.
2. Remove migration sections from both languages.
3. Cross-check headings, examples, and semantic content for alignment.

### Task 8: Full verification and diff review

**Files:** all touched files.

1. Run `cargo fmt --check` in each affected crate.
2. Run `cargo test --all-targets --all-features` in `rs-spi`, `rs-fs`, `rs-mime`, and `rs-magika`.
3. Run `cargo clippy --all-targets --all-features -- -D warnings` in each affected crate.
4. Run `cargo doc --no-deps --all-features` in each affected crate.
5. Review `git diff --check`, per-repository status, and diffs to ensure unrelated user changes remain intact and no SPI re-export remains in `rs-mime`.
