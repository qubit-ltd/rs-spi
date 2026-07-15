# rs-spi Contract Hardening and Downstream Alignment Design

## Goal

Harden the public contracts of `rs-spi`, remove an unnecessary public registration container, make resolver state observable without duplicating registries downstream, improve structured error diagnostics with `thiserror`, and align the directly affected `rs-mime` and `rs-magika` integrations and documentation.

## Scope

- `rs-spi`: registration API internals, resolver accessors and traits, error representation, contract tests, and English/Chinese READMEs.
- `rs-mime`: stop re-exporting `qubit-spi` items, remove duplicated registry ownership where practical, and update stale SPI documentation.
- `rs-magika`: import SPI types from `qubit-spi`, preserve initialization error sources, and restore then migrate the behavior tests that were removed during the uncommitted SPI 0.4 migration.
- `rs-fs`: adapt its resolver wrapper to the new read-only registry access where needed while preserving the existing uncommitted migration.

The general workspace-wide re-export policy is recorded as an architectural rule, but this change only edits the crates directly involved in the `rs-spi` assessment. It does not opportunistically refactor unrelated crates.

## Registration API

`ProviderRegistration<S>` is not a useful public domain abstraction. It only combines a descriptor and provider, performs no validation, and cannot be consumed through a public API independently of `ProviderRegistryBuilder`. The builder already exposes the complete registration operations.

The public `ProviderRegistration` export and module will be removed. The builder will retain:

```rust
pub fn register<P>(
    &mut self,
    descriptor: ProviderDescriptor,
    provider: P,
) -> Result<(), RegistrationError>
where
    P: ServiceProvider<S>;

pub fn register_shared(
    &mut self,
    descriptor: ProviderDescriptor,
    provider: Arc<dyn ServiceProvider<S>>,
) -> Result<(), RegistrationError>;
```

Internally the builder stores a private entry containing the descriptor and shared provider. Both public methods converge on one private atomic validation-and-insert path. Descriptor selector conflicts are fully validated before mutating either the selector ownership map or the registration list.

## Resolver Ownership and Observation

`ProviderResolver<S>` remains an owning runtime facade over an immutable `ProviderRegistry<S>` plus `FallbackPolicy`. It gains:

```rust
pub fn registry(&self) -> &ProviderRegistry<S>;
pub const fn fallback_policy(&self) -> FallbackPolicy;
```

It also implements `Clone` and a bounded `Debug` representation that reports registry metadata and fallback policy without requiring provider implementations to implement `Debug`.

Downstream registry facades should store only the resolver when their separate registry field exists solely for provider enumeration. Enumeration uses `resolver.registry()`. This removes duplicate handles and one potential source of drift while retaining immutable shared registry semantics. No `Default` implementation is added because neither a registry nor a fallback policy has a universally correct default.

## Error Model

`thiserror` remains a dependency and becomes the implementation mechanism for the public error types.

Public error APIs remain opaque structs with stable accessor methods. Private representation enums carry the individual variants and derive `thiserror::Error`; public wrappers derive a transparent error implementation. This keeps representation flexibility while removing repetitive manual `Display` and `Error` implementations.

Diagnostics are strengthened as follows:

- Invalid raw selector input gets an explicit resolution-error variant that retains the original input and the selector validation error as its source.
- Duplicate registration messages identify the conflicting selector, its existing owner, and the provider that attempted to claim it.
- Provider initialization failures retain their concrete source error through `ProviderError::initialization_failed_with_source`.
- Attempt failures expose an explicit failure kind, so callers do not infer “unknown provider” versus “provider error” from combinations of optional fields.
- Existing public accessors and error classifications are preserved where they remain semantically correct.

## Downstream Boundaries

`qubit-mime` will no longer re-export types owned by `qubit-spi`. Callers import MIME-domain types from `qubit-mime` and SPI infrastructure from `qubit-spi`.

`qubit-magika` keeps its own provider and descriptor factory. Assembly remains explicit:

```rust
builder.register(
    magika_mime_detector_descriptor(),
    MagikaMimeDetectorProvider,
)?;
```

Magika initialization converts backend failures with a source-preserving provider error constructor instead of converting the backend error to a string.

## Tests

`rs-spi` contract tests cover:

- owned and shared registration paths;
- atomic behavior after duplicate canonical IDs and aliases;
- registration order and automatic priority ordering, including deterministic ID tie-breaking;
- `Auto`, `Named`, and `Chain` resolution under both fallback policies;
- alias de-duplication within chains;
- invalid selector diagnostics and error sources;
- explicit attempt-failure classification;
- resolver accessors, cloning, and debug output;
- concurrent reads and service creation from cloned immutable registries/resolvers.

For `rs-magika`, the two modified behavior-test files are first restored exactly from `HEAD` with `git restore --source=HEAD -- ...`, then migrated to the current explicit builder and direct SPI imports. Behavioral assertions remain intact; tests may still skip backend-dependent assertions when the optional runtime is unavailable, as the original tests did.

## Documentation

- Update `rs-mime` rustdoc to describe the current split between descriptors, provider factories, registry builders, and resolvers.
- Update both `rs-spi` READMEs to match the final public API and keep their content aligned.
- Remove migration instructions from both `rs-spi` READMEs.

## Compatibility

Removing public `ProviderRegistration` and removing `qubit-mime` SPI re-exports are intentional breaking changes. Resolver additions and improved diagnostics are additive except where callers match exact error display strings. No compatibility shim will be added for APIs that contradict the desired ownership boundaries.
