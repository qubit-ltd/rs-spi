# rs-spi Downstream-Driven Maintenance Implementation Plan

> **For Codex:** Follow this plan task by task with test-first checkpoints. Do
> not alter the existing copyright headers. Preserve all public paths and
> downstream behavior.

**Goal:** Remove proven selector lookup allocation, tighten API guidance and
documentation, complete the bilingual README structure, and restore the
repository's external-test mapping without breaking `qubit-spi` 0.8 users.

**Architecture:** Keep `ProviderSelector` as the owned boundary value while
allowing the registry's selector index to borrow canonical `str` keys. Public
raw-string APIs take an allocation-free path only when the input already
satisfies canonical syntax; normalization and error construction retain the
existing owned path. Test-only domain types move into mirrored fixture modules,
and private production behavior remains tested exclusively through public APIs.

**Tech stack:** Rust 2024, `HashMap` borrowed-key lookup, Criterion 0.8,
integration tests, compile-fail rustdoc, repository CI/alignment scripts.

---

## Task 1: Establish selector allocation evidence

**Files:**

- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Create: `benches/provider_selector_lookup.rs`
- Create: `benches/support/mod.rs`
- Create: `benches/support/tracking_allocator.rs`
- Create: `tests/fixtures/mod.rs`
- Create: `tests/fixtures/tracking_allocator.rs`
- Modify: `tests/mod.rs`
- Modify: `tests/provider_registry_tests.rs`
- Modify: `tests/provider_resolver_tests.rs`

1. Add a test-only tracking allocator that counts allocations only while a
   thread-local guard is active. Install it as the integration test binary's
   global allocator without changing library allocation behavior.
2. Add focused regression tests proving that repeated canonical known-selector
   calls to `ProviderRegistry::find`, `ProviderRegistry::resolve`, and
   `ProviderResolver::create_named` currently allocate. Express the desired
   contract as zero allocations around only the lookup/create call.
3. Run the focused tests and record the expected failure before changing
   production code:

   ```sh
   cargo test --test integration_tests provider_registry_tests::canonical -- --exact
   cargo test --test integration_tests provider_resolver_tests::canonical -- --exact
   ```

4. Add Criterion 0.8 as a dev dependency and an explicit harness-free
   `provider_selector_lookup` benchmark. Benchmark canonical `find`, `resolve`,
   and `create_named`, plus a mixed-case normalization control. Include a
   preflight allocation assertion so benchmark output cannot hide regression.
5. Run the benchmark once to capture the baseline and confirm the allocation
   assertion fails before the implementation:

   ```sh
   cargo bench --bench provider_selector_lookup --no-run
   cargo bench --bench provider_selector_lookup
   ```

## Task 2: Implement the compatibility-preserving borrowed lookup

**Files:**

- Modify: `src/provider_selector.rs`
- Modify: `src/provider_registry.rs`
- Modify: `src/provider_resolver.rs`
- Modify: `src/internal/registry_inner.rs`
- Test: `tests/provider_registry_tests.rs`
- Test: `tests/provider_resolver_tests.rs`
- Test: `benches/provider_selector_lookup.rs`

1. Implement `Borrow<str>` for `ProviderSelector`; ensure its borrowed value has
   the same equality and hash semantics as the derived owned key.
2. Add an internal borrowed selector-index lookup accepting `&str`.
3. In raw public lookup and creation methods, branch on the existing canonical
   token predicate. Resolve a canonical registered selector directly by
   borrowed key. Preserve the current parsing path for whitespace, uppercase,
   invalid input, and owned unknown-provider diagnostics.
4. Keep `ProviderSelection` and explicit `ProviderSelector` paths unchanged.
5. Run the previously failing allocation tests; they must now pass while the
   normalization and error regression tests remain green:

   ```sh
   cargo test --test integration_tests provider_registry_tests
   cargo test --test integration_tests provider_resolver_tests
   ```

6. Run the benchmark and compare canonical paths with the baseline. Retain the
   fast path only if canonical calls reach zero measured allocations and do not
   show a meaningful throughput regression; otherwise revert the production
   optimization but keep the benchmark and accurate performance documentation.

## Task 3: Add `must_use` API contracts test-first

**Files:**

- Modify: `src/created_service.rs`
- Modify: `src/resolved_provider.rs`

1. Add compile-fail rustdoc examples using `#![deny(unused_must_use)]` for a
   discarded `CreatedService`, `ResolvedProvider`, `into_service()` result, and
   `into_parts()` result. Build each value through the public registry/resolver
   API so no visibility is widened for tests.
2. Run doctests and confirm the new `compile_fail` examples unexpectedly compile
   before the attributes are added:

   ```sh
   cargo test --doc
   ```

3. Add type-level `#[must_use]` to `CreatedService` and `ResolvedProvider`, and
   method-level `#[must_use]` to `into_service` and `into_parts` with concise
   diagnostics where useful.
4. Rerun `cargo test --doc` and confirm all compile-fail contracts pass.

## Task 4: Correct inline and rustdoc consistency

**Files:**

- Modify: `src/**/*.rs`

1. Run `./style-check.sh` and retain the relevant pre-change failures as the
   static red checkpoint.
2. Replace every exact `# Arguments` rustdoc heading in `src/` with
   `# Parameters` without rewriting unrelated prose.
3. Add field documentation to the public tuple field of `ProviderSelector` and
   every other tuple field identified by the style check.
4. Apply the inline decision table:
   - add `#[inline(always)]` to the pure forwarding
     `ProviderResolver::create_chain`;
   - add `#[inline(always)]` to the private forwarding
     `ResolutionError::no_provider_succeeded`;
   - keep `#[inline]` on the short forwarding constructors `exhausted` and
     `stopped_by_policy`;
   - remove `#[inline]` from iterative `ResolutionError::is_absence`.
5. Re-run `./style-check.sh` and fix only in-scope findings.

## Task 5: Complete bilingual README structure

**Files:**

- Modify: `README.md`
- Modify: `README.zh_CN.md`

1. Add the repository's coverage badge and remove the non-template
   documentation badge so each README has the authoritative six-badge block.
2. Append the exact final four sections from the style reference in order:
   Testing, License, Contributing, Author; use their Chinese counterparts in
   `README.zh_CN.md`.
3. Resolve all badge links and placeholders from this repository's Cargo and
   Git metadata. Point testing instructions at the actual scripts and license
   text at `LICENSE`.
4. Run `./style-check.sh` and inspect the README diff for bilingual structural
   parity.

## Task 6: Extract reusable test fixtures one type per file

**Files:**

- Create/modify: `tests/fixtures/mod.rs`
- Create: one snake-case fixture file per existing test-only struct, enum, or
  trait moved from `tests/**/*_tests.rs`
- Modify: all existing `tests/**/*_tests.rs` that currently declare fixtures

1. Move each existing test-only `struct`, `trait`, and `enum` into its own file
   under `tests/fixtures/`, preserving behavior and using the minimum
   `pub(crate)` visibility needed by integration tests.
2. Consolidate genuinely identical fixtures such as the duplicate text service
   specification. Keep semantically different providers separate even when
   their old names matched.
3. Add the authoritative header to each new file; do not change accepted
   headers in existing files.
4. Update test imports without changing assertions.
5. Run the complete existing integration suite before adding new mapped tests:

   ```sh
   cargo test --test integration_tests
   ```

## Task 7: Restore source-to-test module mapping

**Files:**

- Create: `tests/error/provider_descriptor_error_tests.rs`
- Create: `tests/error/provider_error_kind_tests.rs`
- Create: `tests/error/provider_id_error_tests.rs`
- Create: `tests/error/provider_selection_error_tests.rs`
- Create: `tests/error/provider_selector_error_tests.rs`
- Create: `tests/fallback_policy_tests.rs`
- Create: `tests/provider_selection_kind_tests.rs`
- Create: `tests/resolved_provider_tests.rs`
- Create: `tests/internal/mod.rs`
- Create: `tests/internal/builder_entry_tests.rs`
- Create: `tests/internal/provider_selection_repr_tests.rs`
- Create: `tests/internal/registry_entry_tests.rs`
- Create: `tests/internal/registry_inner_tests.rs`
- Modify: `tests/error/mod.rs`
- Modify: `tests/mod.rs`

1. Add mapped external test modules for every currently unmapped public source
   module. Cover public construction, display/source behavior, discriminants,
   and boundary cases rather than duplicating implementation details.
2. Add mapped `tests/internal/*` modules that exercise each private component's
   observable contract through public registry, selection, and resolver APIs.
   Do not import private modules or widen production visibility.
3. Declare every module through the existing single integration-test entry
   point.
4. Run each new group, then the full integration suite:

   ```sh
   cargo test --test integration_tests error
   cargo test --test integration_tests fallback_policy_tests
   cargo test --test integration_tests provider_selection_kind_tests
   cargo test --test integration_tests resolved_provider_tests
   cargo test --test integration_tests internal
   cargo test --test integration_tests
   ```

## Task 8: Full repository and downstream verification

**Files:**

- Inspect: all changed files
- Do not modify: downstream `Cargo.lock` files

1. Run the repository-prescribed sequence in exact order:

   ```sh
   ./align-ci.sh
   ./ci-check.sh
   ```

2. Inspect changes made by alignment, run `git diff --check`, and rerun focused
   checks if alignment touched Rust sources.
3. Run `./coverage.sh json` only if CI reports coverage below its threshold;
   add meaningful boundary/error tests for in-scope uncovered behavior and
   rerun the sequence.
4. Run the final benchmark and record allocation and timing results:

   ```sh
   cargo bench --bench provider_selector_lookup
   ```

5. Check the reviewed downstream crates against the local `rs-spi` only where
   their existing manifests already support local wiring. Preserve their dirty
   lockfiles and do not use commands that rewrite them. At minimum use
   `cargo check --locked` when it consumes the local crate without mutation.
6. Review `git status --short`, `git diff --stat`, and the complete diff. Confirm
   only the accepted items 1–4, 6, and 7 are present and the file-header finding
   remains intentionally unchanged.
