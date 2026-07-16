# rs-spi Error API Cleanup Design

**Status:** Approved in conversation on 2026-07-16
**Scope:** `qubit-spi` and its direct workspace consumers `qubit-fs`,
`qubit-mime`, and `qubit-magika`

## Goal

Finish the error API simplification begun for `ResolutionError` and
`AttemptFailure`: expose each validation or registration failure through one
public non-exhaustive enum, remove parallel discriminator types and
variant-specific optional accessors, migrate direct consumers, and bring the
filesystem provider documentation in line with `qubit-spi` 0.7.

Selector lookup allocation is explicitly outside this change.

## Design Principles

- A public error has one discriminated representation: its non-exhaustive enum.
- Callers that need variant-specific fields match the enum directly with a
  wildcard arm and `..` for forward compatibility.
- A query method remains only when it expresses one meaningful value across
  every current variant, rather than returning `Option` to reveal which variant
  is stored.
- `ProviderErrorKind` remains because it is an input to resolver fallback
  policy, not a duplicate inspection API.
- `ProviderSelectionKind` and `ResolutionTermination` remain because they
  classify non-error domain state and resolution outcomes respectively.

## Public API Changes

The following public types and their source files are removed:

- `ProviderDescriptorErrorKind`
- `ProviderIdErrorKind`
- `ProviderSelectionErrorKind`
- `ProviderSelectorErrorKind`
- `RegistrationErrorKind`

The corresponding `kind()` methods are removed from their error enums.

The following variant-specific optional or predicate accessors are removed:

- `ProviderDescriptorError::alias_index()`
- `ProviderDescriptorError::selector_error()`
- `ProviderSelectionError::selector_index()`
- `ProviderSelectionError::selector_input()`
- `ProviderSelectionError::selector_error()`
- `ProviderSelectionError::is_empty_chain()`
- `ProviderSelectorError::normalized()`
- `ProviderSelectorError::is_empty()`

The following accessors remain because they return a meaningful value for every
current variant:

- `ProviderDescriptorError::alias()`
- `ProviderIdError::input()`
- `ProviderSelectorError::input()`

`RegistrationError` currently has one variant. Its `selector()`,
`existing_provider()`, and `provider()` accessors are removed; callers match
`RegistrationError::DuplicateSelector` directly. This avoids maintaining both
field access and pattern-matching APIs as more registration failures are added.

All affected enums remain `#[non_exhaustive]`. Direct consumers must therefore
include a wildcard arm and use `..` when matching struct variants.

## Downstream Migration

### qubit-mime

The detector and classifier registration adapters match
`RegistrationError::DuplicateSelector` and read `selector` directly.

MIME configuration error adapters match `ProviderSelectionError` directly:

- `InvalidSelector` maps its `selector_input` to the corresponding invalid
  detector or classifier name error.
- `EmptyChain` maps to the corresponding empty-name error.
- A wildcard arm maps future variants to `NoAvailableDetector` or
  `NoAvailableClassifier` with the SPI error's `Display` text. This avoids
  inventing a provider name that a future variant may not contain.

Resolution adapters distinguish an empty selector source with
`matches!(source, ProviderSelectorError::Empty { .. })` instead of calling
`is_empty()`.

### qubit-fs

Production code already depends only on `ResolutionError::is_absence()` and
requires no error API migration.

The English and Chinese user-guide provider examples are rewritten for the
0.7 contract:

- import `ProviderError` from `qubit_spi::error`;
- implement only `ServiceProvider<FileSystemSpec>::create`;
- return `Arc<dyn FileSystem>` as selected by `FileSystemSpec`;
- construct `ProviderDescriptor` from a validated `ProviderId`;
- register through `FileSystemRegistryBuilder` before building the immutable
  runtime registry.

### qubit-magika

No production migration is expected because it uses only `ProviderError`,
`ProviderDescriptor`, `ProviderId`, and `ServiceProvider`. Its checks still run
to detect accidental public-contract fallout.

## Style Alignment

`ProviderRegistry::builder()` is a pure forwarding method and changes from
`#[inline]` to `#[inline(always)]` according to the repository inline policy.
No other style-only changes are included.

## Testing

Implementation follows test-first removal:

1. Add a `compile_fail` migration example to `qubit_spi::error` that imports all
   five obsolete Kind types. It fails its doctest before the removal because
   the imports still compile, then passes after the types are deleted.
2. Rewrite `rs-spi` behavioral tests to require direct enum matching and remove
   modules dedicated to the deleted Kind types.
3. Remove the five Kind types, `kind()` methods, and variant-specific accessors;
   keep the three cross-variant accessors.
4. Migrate `qubit-mime` adapters and tests to direct matching.
5. Update the two `qubit-fs` user guides and verify their provider snippets
   against the current types and method signatures.
6. Apply the forwarding inline attribute change.

The public behavior of validation, registration, resolution, fallback, and
error source chaining does not change. Tests continue to cover every existing
error variant and retained cross-variant query.

## Verification

For each modified repository, run its prescribed commands from that repository
root in this order:

1. `./align-ci.sh`
2. `./ci-check.sh`
3. only if CI reports coverage below threshold, `./coverage.sh json`

Because the repositories are independent Git repositories, their changes and
verification results remain separate. No commit or push is part of this task
unless separately authorized.
