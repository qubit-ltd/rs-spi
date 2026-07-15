# rs-spi Error Boundary Redesign

## Goal

Replace the overloaded provider error model with lifecycle-specific public
errors, make provider selections invariant-safe, expose raw-selector resolver
entry points, improve aggregate diagnostics, and migrate every current
`qubit-spi` downstream without preserving source compatibility.

The redesign also adds the small missing ownership and collection APIs
`CreatedService::into_parts()` and `ProviderRegistry::len()`.

## Scope

The change covers:

- `rs-spi`: identifier, selector, descriptor, registration, selection, and
  resolution APIs; diagnostics; documentation; and contract tests.
- `rs-fs`: filesystem-provider assembly, runtime selector handling, and SPI
  error translation.
- `rs-mime`: detector and classifier assembly, runtime selector handling, and
  SPI error translation.
- `rs-magika`: descriptor construction and explicit MIME-provider assembly.

Backward compatibility is not a requirement. Removed names and changed return
types do not receive deprecated aliases or compatibility shims.

## Design Principles

1. Each public error type describes one lifecycle boundary.
2. Invalid states are prevented by construction where practical.
3. Resolver methods own runtime parsing and translate it into resolution
   errors.
4. Structured diagnostics remain available independently of formatted text.
5. The SPI core remains explicitly assembled, synchronous, immutable after
   build, and free of global provider discovery.

## Error Boundaries

### ProviderIdError

`ProviderId::new` returns `ProviderIdError`.

Its public kind has two variants:

- `Empty`: the supplied canonical ID is empty.
- `NonCanonical`: the supplied value violates canonical ID syntax.

The error retains the verbatim input when one exists. It does not normalize
the value.

### ProviderSelectorError

`ProviderSelector::parse` returns `ProviderSelectorError`.

Its public kind has two variants:

- `Empty`: trimming leaves no selector.
- `Invalid`: the normalized selector violates selector syntax.

The error retains both the verbatim input and, for `Invalid`, the normalized
value. Selector parsing continues to trim surrounding whitespace and lowercase
ASCII letters before validation.

### ProviderDescriptorError

`ProviderDescriptor::with_aliases` returns `ProviderDescriptorError`.

Its public kind has three variants:

- `InvalidAlias`: an alias cannot be parsed as a provider selector.
- `DuplicateAlias`: two aliases normalize to the same selector.
- `AliasMatchesId`: an alias normalizes to the descriptor's canonical ID.

Invalid aliases retain their zero-based input index, verbatim input, and
`ProviderSelectorError` source. Duplicate errors retain the normalized alias.
No descriptor mutation is observable on failure because the method consumes
and returns the descriptor.

### RegistrationError

`ProviderRegistryBuilder::register` and `register_shared` return
`RegistrationError` only for conflicts between registry entries.

Its public kind has one variant:

- `DuplicateSelector`: a canonical ID or alias is already owned by a provider
  accepted by this builder.

The error retains the conflicting selector, existing owner, and prospective
owner. Registration remains atomic: all selectors are validated before the
builder records any part of the new provider.

### ProviderSelectionError

`ProviderSelection::named` and `ProviderSelection::chain` return
`ProviderSelectionError`.

Its public kind has two variants:

- `InvalidSelector`: one selection input cannot be parsed.
- `EmptyChain`: a chain contains no selector inputs.

An invalid selector retains the zero-based position, verbatim input, and
`ProviderSelectorError` source. Named selections use position zero.

### ResolutionError

Resolver operations return `ResolutionError` with these public kinds:

- `InvalidSelector`: a raw-selector resolver entry point receives an invalid
  selector.
- `EmptySelection`: a raw chain contains no inputs.
- `UnknownProvider`: a valid named selector resolves to no provider.
- `EmptyRegistry`: automatic selection is requested from an empty registry.
- `NoProviderSucceeded`: at least one named, automatic, or chained candidate
  was considered but no service was created.

`InvalidSelector` retains the raw input, optional zero-based chain position,
and `ProviderSelectorError` source. `UnknownProvider` retains the normalized
selector. `NoProviderSucceeded` retains ordered `AttemptFailure` records.

An empty raw chain is a resolution error when passed directly to the resolver,
and a selection-construction error when passed to `ProviderSelection::chain`.
This keeps each entry point's error type internally complete.

## Provider Selection Representation

`ProviderSelection` becomes an opaque public struct over a private
representation. Callers cannot construct an empty chain or bypass selector
validation.

Construction APIs are:

```rust
pub const fn auto() -> ProviderSelection;

pub fn named(
    value: impl AsRef<str>,
) -> Result<ProviderSelection, ProviderSelectionError>;

pub fn chain<I, T>(
    values: I,
) -> Result<ProviderSelection, ProviderSelectionError>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>;
```

`Default` produces automatic selection. Read-only observation uses:

```rust
pub const fn kind(&self) -> ProviderSelectionKind;
pub fn selector(&self) -> Option<&ProviderSelector>;
pub fn selectors(&self) -> &[ProviderSelector];
```

`ProviderSelectionKind` is a copyable non-exhaustive enum with `Auto`, `Named`,
and `Chain`. `selector()` returns `Some` only for named selection.
`selectors()` returns the chain slice and an empty slice for other kinds.

## Resolver API

The pre-parsed entry point remains available:

```rust
pub fn create(
    &self,
    selection: &ProviderSelection,
    config: &S::Config,
) -> Result<CreatedService<S::Output>, ResolutionError>;
```

The resolver adds raw-input conveniences:

```rust
pub fn create_auto(
    &self,
    config: &S::Config,
) -> Result<CreatedService<S::Output>, ResolutionError>;

pub fn create_named(
    &self,
    selector: impl AsRef<str>,
    config: &S::Config,
) -> Result<CreatedService<S::Output>, ResolutionError>;

pub fn create_chain<I, T>(
    &self,
    selectors: I,
    config: &S::Config,
) -> Result<CreatedService<S::Output>, ResolutionError>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>;
```

Raw methods parse internally and never expose `ProviderSelectionError` or
`RegistrationError`. `create_chain` reports the index of an invalid selector.
The pre-parsed `create` path cannot produce `InvalidSelector` or
`EmptySelection` because `ProviderSelection` enforces its invariants.

Named selection attempts exactly one resolved provider. Automatic and chained
selection continue to obey the resolver's `FallbackPolicy`. Chains continue to
record unknown selectors, skip repeated provider entries reached through
aliases, and preserve encounter order in diagnostics.

## Aggregate Diagnostics

`AttemptFailure` remains an opaque structured diagnostic and implements
`Display` and `Error`.

- Unknown-provider attempts display their requested selector.
- Provider-error attempts display the canonical provider ID, provider error
  kind, and reason.
- `AttemptFailure::source()` continues to expose the retained provider source
  when present.

`ResolutionError::Display` includes an ordered summary for
`NoProviderSucceeded`, rather than only an attempt count. Structured callers
continue to use `attempts()`, `kind()`, `selector_input()`, and related
accessors. The format is intended for diagnostics and is not a stable parsing
protocol.

`ResolutionError` does not fabricate a single standard source for aggregate
failures because multiple attempts may have independent causes.

## Collection and Ownership Completeness

`ProviderRegistry` adds:

```rust
pub fn len(&self) -> usize;
```

`is_empty()` remains and is implemented consistently with `len()`.

`CreatedService` adds:

```rust
pub fn into_parts(self) -> (ProviderId, T);
```

`into_service()` remains for callers that intentionally discard the winning
provider identity.

## Downstream Migration

### rs-fs

`FileSystemRegistry::fs` uses `ProviderResolver::create_named` directly. ID,
descriptor, registration, and resolution failures map through separate domain
conversion functions. Existing filesystem error kinds remain unchanged.

### rs-mime

Detector and classifier registries use raw resolver methods for explicit and
configured selections. Descriptor construction errors, registration conflicts,
selection-construction errors, and runtime resolution failures each have a
dedicated MIME-domain mapping boundary.

Automatic selection from an empty registry is reported with a non-empty,
specific diagnostic. Exhausted provider attempts preserve their individual
reasons without downstream string assembly when the aggregate SPI diagnostic
is sufficient.

### rs-magika

The Magika descriptor factory adapts to `ProviderIdError` and
`ProviderDescriptorError`. Its public assembly model remains descriptor plus
provider, and initialization failures continue to preserve their MIME source.

## Testing Strategy

Implementation follows test-driven development. Each behavior is introduced by
a focused failing integration test before production code changes.

The `rs-spi` contract suite covers:

- complete valid and invalid ID grammar;
- selector normalization and invalid-input diagnostics;
- invalid, repeated, and canonical-ID aliases;
- atomic cross-provider registration conflicts;
- opaque selection construction and observation;
- empty and invalid selection input;
- raw auto, named, and chain resolver entry points;
- invalid raw selector positions;
- empty automatic registry classification;
- fallback policy behavior and alias de-duplication;
- attempt and aggregate formatted diagnostics and retained sources;
- registry length and emptiness;
- service decomposition through `into_parts()`.

Each downstream receives focused regression tests for its migrated selector and
error mapping. Final verification runs formatting, all targets and features,
Clippy with warnings denied, rustdoc, and repository diff checks in `rs-spi`,
`rs-fs`, `rs-mime`, and `rs-magika`.

## Non-Goals

- No compatibility aliases for removed error kinds or public enum variants.
- No automatic provider discovery, global registry, inventory integration, or
  linker registration.
- No asynchronous provider contract.
- No logging facade or provider availability preflight.
- No changes to provider priority or fallback semantics beyond explicit empty
  registry and empty selection errors.
- No unrelated refactoring in downstream crates.
