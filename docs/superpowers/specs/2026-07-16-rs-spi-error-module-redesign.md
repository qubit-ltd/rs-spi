# rs-spi Error Module Redesign

**Status:** Accepted in conversation on 2026-07-16; awaiting written-spec review
**Date:** 2026-07-16
**Scope:** `qubit-spi` and its direct workspace consumers: `qubit-fs`,
`qubit-mime`, and `qubit-magika`

## Context

`qubit-spi` 0.5 uses opaque `ResolutionError` and `AttemptFailure` values.
Callers inspect a kind enum and then query several optional accessors to recover
variant-specific data. The direct consumers repeat this protocol in filesystem,
MIME detector, and media classifier error adapters. Those adapters contain
defensive placeholder values because the type system does not express which
fields belong to each error variant.

The crate also exposes error types from its root while their implementation is
spread across the root source directory and the general-purpose internal
module. Canonical provider IDs are reparsed as selectors in two internal paths,
despite the `ProviderId` invariant already guaranteeing valid selector syntax.

The user permits breaking changes, does not require compatibility aliases, and
approved a single public `qubit_spi::error` namespace.

## Goals

- Make resolution and attempt diagnostics directly and exhaustively
  pattern-matchable.
- Remove the kind-plus-optional-accessor protocol for `ResolutionError` and
  `AttemptFailure`.
- Give all public error types one canonical path below `qubit_spi::error`.
- Keep error implementation details under `src/error/internal/` rather than the
  registry's general internal module.
- Use one public representation for both pattern matching and error storage,
  without a parallel borrowed-view API.
- Preserve complete provider error chains.
- Replace impossible provider-ID reparsing failures with an infallible typed
  conversion.
- Correct `ProviderDescriptor` method order and inline attributes.
- Retain `ProviderSelection` and document its pre-parse-and-reuse use case with
  a runnable Rustdoc example.
- Update every direct workspace consumer to the new paths and structured error
  matching.

## Non-goals

- Redesigning the remaining validation and registration errors as public
  enums.
- Adding compatibility re-exports at the crate root.
- Adding new fallback policies or changing provider selection behavior.
- Adding async provider construction, global registries, runtime discovery, or
  generic downstream registry wrappers.
- Refactoring unrelated downstream domain errors.

## Considered Approaches

### Borrowed public views over opaque storage

Add `ResolutionErrorRef` and `AttemptFailureRef` enums while retaining the
current opaque types. This is additive but creates two representations for each
diagnostic and leaves the old accessor matrix in the public API.

### Public enums with compatibility helpers

Make the errors public enums but retain kind enums, accessors, and root
re-exports. This reduces downstream friction while carrying redundant API
surface indefinitely.

### Direct public enums in one error module

Make `ResolutionError` and `AttemptFailure` public enums, remove their redundant
kind enums and variant accessors, and expose all errors only through
`qubit_spi::error`. This is the selected approach because compatibility is not
required and each diagnostic then has one representation and one public path.

## Public Error Contract

`AttemptFailure` becomes a directly matchable enum:

```rust
pub enum AttemptFailure {
    UnknownProvider {
        requested_selector: ProviderSelector,
    },
    ProviderError {
        requested_selector: Option<ProviderSelector>,
        provider_id: ProviderId,
        error: ProviderError,
    },
}
```

The enum keeps custom `Display` and `Error` implementations. The
`ProviderError` variant returns the retained `ProviderError` from
`Error::source()`, producing the chain `AttemptFailure -> ProviderError ->
provider source`. Unknown-provider display text is generated from the retained
selector rather than stored separately.

`ResolutionError` becomes a directly matchable enum:

```rust
pub enum ResolutionError {
    InvalidSelector {
        input: Box<str>,
        selector_index: Option<usize>,
        source: ProviderSelectorError,
    },
    EmptySelection,
    UnknownProvider {
        selector: ProviderSelector,
    },
    EmptyRegistry,
    NoProviderSucceeded {
        attempts: Box<[AttemptFailure]>,
    },
}
```

The enum keeps custom ordered diagnostics in `Display`. `Error::source()`
returns the selector parsing error for `InvalidSelector`; aggregate failures do
not claim a single causal source.

Public enum variants are necessarily constructible by callers. This is an
intentional consequence of choosing direct exhaustive matching instead of an
opaque value plus a borrowed public view. No memory-safety or registry
correctness property depends on error provenance. The resolver itself continues
to emit `NoProviderSucceeded` only with at least one attempt, but the public
error type does not claim to enforce that invariant for caller-constructed
diagnostics.

The following public types are removed:

- `AttemptFailureKind`
- `ResolutionErrorKind`

The corresponding `kind()` methods and variant-specific optional accessors are
removed. Callers use exhaustive pattern matching instead.

## Public Module Layout

The crate exposes one public error module:

```text
qubit_spi::error
├── AttemptFailure
├── ProviderDescriptorError
├── ProviderDescriptorErrorKind
├── ProviderError
├── ProviderErrorKind
├── ProviderIdError
├── ProviderIdErrorKind
├── ProviderSelectionError
├── ProviderSelectionErrorKind
├── ProviderSelectorError
├── ProviderSelectorErrorKind
├── RegistrationError
├── RegistrationErrorKind
└── ResolutionError
```

`lib.rs` declares `pub mod error;` and does not re-export these types from the
crate root. Service, identity, descriptor, registry, resolver, selection, and
fallback types remain at their existing root paths.

The source layout is:

```text
src/error/
├── mod.rs
├── attempt_failure.rs
├── provider_descriptor_error.rs
├── provider_descriptor_error_kind.rs
├── provider_error.rs
├── provider_error_kind.rs
├── provider_id_error.rs
├── provider_id_error_kind.rs
├── provider_selection_error.rs
├── provider_selection_error_kind.rs
├── provider_selector_error.rs
├── provider_selector_error_kind.rs
├── registration_error.rs
├── registration_error_kind.rs
├── resolution_error.rs
└── internal/
    ├── mod.rs
    ├── provider_descriptor_error_repr.rs
    ├── provider_id_error_repr.rs
    ├── provider_selection_error_repr.rs
    ├── provider_selector_error_repr.rs
    └── registration_error_repr.rs
```

The following obsolete files are deleted:

- `src/attempt_failure_kind.rs`
- `src/resolution_error_kind.rs`
- `src/internal/attempt_failure_repr.rs`
- `src/internal/resolution_error_repr.rs`

The root `src/internal/` directory retains only builder, registry, and provider
selection implementation types.

## Provider ID Conversion

`ProviderSelector` implements an infallible conversion from a borrowed canonical
ID:

```rust
impl From<&ProviderId> for ProviderSelector
```

The conversion copies canonical text directly into selector storage. Registry
registration and descriptor alias validation use this conversion instead of
calling `ProviderSelector::parse(...).expect(...)`. Their impossible panic
documentation is removed.

## ProviderDescriptor Organization

`ProviderDescriptor` keeps its behavior and public signatures. Its inherent
methods are ordered by constructor status, visibility, and functional
adjacency:

1. `new`
2. `id`
3. `aliases`
4. `with_aliases`
5. `priority`
6. `with_priority`

`with_aliases` has iterative validation and receives no inline attribute.
`with_priority` is an extremely thin setter-style transformation and uses
`#[inline(always)]`.

## ProviderSelection Documentation

`ProviderSelection` remains an opaque invariant-preserving value. Its type-level
Rustdoc gains a runnable example that:

1. assembles a small typed registry;
2. parses a named selection once;
3. calls `ProviderResolver::create(&selection, config)` more than once;
4. demonstrates why callers may prefer a reusable selection over the raw
   `create_named` convenience method.

The example uses the crate's synchronous provider contract and participates in
normal doctest execution.

## Downstream Migration

`qubit-fs`, `qubit-mime`, and `qubit-magika` import error values from
`qubit_spi::error`. Root imports remain only for non-error SPI types.

Filesystem resolution maps `ResolutionError` through direct variant matching.
`NoProviderSucceeded` inspects its attempts by matching
`AttemptFailure::ProviderError`; unknown attempts cannot accidentally satisfy
the provider-absence predicate.

MIME detector and media classifier adapters directly match:

- `InvalidSelector` and its concrete parser source;
- `UnknownProvider` and its concrete selector;
- singleton `NoProviderSucceeded` attempts;
- aggregate or empty failures.

They no longer use `"<invalid>"` or `"<unknown>"` placeholders. A singleton
unknown chain attempt remains a domain-level unknown detector. Provider errors
retain their current unavailable/backend mapping.

`qubit-magika` updates only affected import paths unless compilation reveals a
real structured-error use.

Pre-existing user changes in `rs-mime/Cargo.toml`, `rs-mime/Cargo.lock`, and
`rs-magika/Cargo.lock` are preserved and are not reverted or reformatted as
part of this work.

## Testing Strategy

Implementation follows test-driven development:

1. Change `rs-spi` tests to pattern-match the proposed public enum variants and
   run a focused test to observe the expected compile failure.
2. Add a `From<&ProviderId>` behavior test and observe its compile failure.
3. Implement the enum and conversion changes until focused tests pass.
4. Update direct downstream tests to exercise structured domain mapping,
   including invalid, unknown, singleton unknown-chain, provider failure, and
   exhausted aggregate cases.
5. Add or update Rustdoc coverage for reusable `ProviderSelection`.
6. Run each affected repository's prescribed alignment and CI sequence.

Tests remain in each crate's external `tests/` tree. No production visibility is
widened solely for testing.

## Verification

For each affected repository, run its scripts from that repository root:

1. `./align-ci.sh`
2. `./ci-check.sh`
3. only when CI reports coverage below its threshold, `./coverage.sh json`

After each formatting or alignment command, inspect the resulting changes.
Run coverage-directed test work only when the repository-prescribed CI result
requires it. Do not claim compatibility with consumers outside the inspected
workspace; this is an intentional breaking redesign.
