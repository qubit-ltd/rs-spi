# Qubit SPI User Guide

This guide covers the complete public usage model of `qubit-spi` 0.8.

## Overview

Qubit SPI provides typed infrastructure for applications that support multiple
implementations of the same service. The application defines a service family,
registers provider factories during startup, builds an immutable registry, and
then resolves services through explicit selection rules.

The crate deliberately has no global registry or discovery side effects.
Applications decide which providers are linked, when they are registered, and
which registry or resolver is shared with each subsystem. This keeps startup
failures visible and makes tests independent of process-wide state.

## Installation

Add the crate to `Cargo.toml`:

```toml
[dependencies]
qubit-spi = "0.8"
```

Version 0.8 requires Rust 1.94 or later. The crate has no feature flags and its
only runtime dependency is `thiserror`.

Most applications import core types from `qubit_spi` and error types from
`qubit_spi::error`:

```rust
use qubit_spi::error::{ProviderError, ResolutionError};
use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderResolver,
    ProviderSelection,
    ServiceProvider,
    ServiceSpec,
};
```

## Core Model

The main types form a startup-to-runtime pipeline:

| Stage | Type | Responsibility |
| --- | --- | --- |
| Service definition | `ServiceSpec` | Binds a configuration type to the complete output type returned by every provider in one service family. |
| Provider implementation | `ServiceProvider<S>` | Creates `S::Output` from `&S::Config` and classifies creation failures. |
| Registration metadata | `ProviderDescriptor` | Holds a canonical ID, aliases, and automatic-selection priority. |
| Startup assembly | `ProviderRegistryBuilder<S>` | Registers factories and rejects selector conflicts before a registry is built. |
| Runtime catalog | `ProviderRegistry<S>` | Provides immutable lookup and deterministic automatic ordering. |
| Selection | `ProviderSelection` | Represents a validated automatic, named, or chained request. |
| Creation | `ProviderResolver<S>` | Applies a selection and fallback policy to create a service. |
| Success | `CreatedService<S::Output>` | Returns the output together with the canonical ID of the winning provider. |

Provider identity belongs to registration, not to the factory object. The same
factory type can therefore be registered differently in distinct registries.
The SPI core returns exactly the output type selected by `ServiceSpec`; it does
not insert or remove `Box`, `Arc`, or `Rc` wrappers.

## Defining a Service

Define one marker type implementing `ServiceSpec` for each independent service
family:

```rust
use std::sync::Arc;

use qubit_spi::ServiceSpec;

trait Greeter: Send + Sync {
    fn greet(&self, name: &str) -> String;
}

struct GreeterConfig {
    prefix: String,
}

struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = GreeterConfig;
    type Output = Arc<dyn Greeter>;
}
```

`Config` may be unsized, so a provider can accept a view such as `str` or a
trait object. `Output` is the complete value returned to the caller. Choose an
owned value, `Box<dyn Trait>`, `Arc<dyn Trait>`, or another handle according to
the service's actual ownership and concurrency requirements.

## Implementing Providers

Each provider implements `ServiceProvider<S>`. Provider implementations must be
`Send + Sync + 'static`, because registries retain and may share them. The
configuration is borrowed and the output is created for each call.

```rust
use std::sync::Arc;

use qubit_spi::error::ProviderError;
use qubit_spi::ServiceProvider;

# trait Greeter: Send + Sync {
#     fn greet(&self, name: &str) -> String;
# }
# struct GreeterConfig { prefix: String }
# struct GreeterSpec;
# impl qubit_spi::ServiceSpec for GreeterSpec {
#     type Config = GreeterConfig;
#     type Output = Arc<dyn Greeter>;
# }
struct LocalGreeter {
    prefix: String,
}

impl Greeter for LocalGreeter {
    fn greet(&self, name: &str) -> String {
        format!("{} {name}", self.prefix)
    }
}

struct LocalProvider;

impl ServiceProvider<GreeterSpec> for LocalProvider {
    fn create(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderError> {
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "the greeting prefix must not be empty",
            ));
        }
        Ok(Arc::new(LocalGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}
```

Use the most accurate `ProviderError` constructor. The classification directly
controls whether a resolver is allowed to try another provider.

## Provider Identity and Metadata

`ProviderId` is a strict canonical identity. It must be lowercase ASCII, begin
and end with an ASCII alphanumeric character, and contain only alphanumeric
characters plus `-`, `_`, `.`, and `+`. It is not trimmed or normalized:

```rust
use qubit_spi::ProviderId;

let id = ProviderId::new("local-v2")?;
assert_eq!("local-v2", id.as_str());
# Ok::<(), qubit_spi::error::ProviderIdError>(())
```

`ProviderSelector` is intended for configuration and user input. Parsing trims
surrounding whitespace and lowercases ASCII letters before applying the same
token grammar. Thus `" LOCAL-V2 "` selects the canonical ID `local-v2`.

A `ProviderDescriptor` combines the canonical ID with aliases and priority:

```rust
use qubit_spi::{ProviderDescriptor, ProviderId};

let descriptor = ProviderDescriptor::new(ProviderId::new("local")?)
    .with_aliases(["builtin", "default"])?
    .with_priority(50);

assert_eq!("local", descriptor.id().as_str());
assert_eq!(50, descriptor.priority());
# Ok::<(), Box<dyn std::error::Error>>(())
```

Aliases are parsed as selectors. A descriptor rejects an invalid alias, an
alias equal to its canonical ID, and duplicate aliases after normalization.
Priority affects automatic selection only; named and chained selection retain
the caller's requested target or order.

## Building a Registry

Build registries during application startup:

```rust
# use std::sync::Arc;
# use qubit_spi::error::ProviderError;
use qubit_spi::{ProviderDescriptor, ProviderId, ProviderRegistry};
# trait Greeter: Send + Sync { fn greet(&self, name: &str) -> String; }
# struct GreeterConfig { prefix: String }
# struct GreeterSpec;
# impl qubit_spi::ServiceSpec for GreeterSpec {
#     type Config = GreeterConfig;
#     type Output = Arc<dyn Greeter>;
# }
# struct LocalProvider;
# impl qubit_spi::ServiceProvider<GreeterSpec> for LocalProvider {
#     fn create(&self, _: &GreeterConfig) -> Result<Arc<dyn Greeter>, ProviderError> {
#         Err(ProviderError::unavailable("example provider"))
#     }
# }

let mut builder = ProviderRegistry::<GreeterSpec>::builder();
builder.register(
    ProviderDescriptor::new(ProviderId::new("local")?)
        .with_aliases(["builtin"])?
        .with_priority(50),
    LocalProvider,
)?;
let registry = builder.build();

assert_eq!(1, registry.len());
assert!(!registry.is_empty());
# Ok::<(), Box<dyn std::error::Error>>(())
```

`register` accepts an owned concrete provider and stores it in shared registry
storage. Use `register_shared` when the factory is already held as
`Arc<dyn ServiceProvider<S>>`.

Every canonical ID and alias must be unique across the registry. Registration
checks all selector claims before mutating the builder, so a rejected
registration does not reserve a partial set of aliases. `RegistrationError`
reports the conflicting selector, its existing owner, and the provider that
attempted the new claim.

After `build`, the registry is immutable. `clone` only clones its internal
`Arc`, so registry handles are inexpensive to share. `descriptors()` and
`provider_ids()` iterate in registration order. `find` returns `None` for both
invalid and unknown input; use `resolve` when callers need a structured
`ResolutionError` that distinguishes these cases.

## Selecting Providers

Qubit SPI supports three selection modes:

- `ProviderSelection::auto()` uses the registry's deterministic automatic
  order: priority descending, then canonical provider ID ascending.
- `ProviderSelection::named(value)` validates one selector and tries exactly
  that provider.
- `ProviderSelection::chain(values)` validates a nonempty ordered selector
  list and tries candidates in input order.

```rust
use qubit_spi::ProviderSelection;

let automatic = ProviderSelection::auto();
let named = ProviderSelection::named(" local ")?;
let chain = ProviderSelection::chain(["cloud", "local"])?;

assert!(named.selector().is_some());
assert_eq!(2, chain.selectors().len());
# Ok::<(), qubit_spi::error::ProviderSelectionError>(())
```

During chain resolution, unknown selectors are recorded as failed attempts and
resolution continues. If an ID and one of its aliases both appear in the same
chain, the underlying provider is invoked only once. Named selection never
falls back, even when the resolver uses `OnAnyError`.

## Fallback Policies

`FallbackPolicy` applies after a provider factory returns `ProviderError` in
automatic or chained selection:

| Policy | `Unsupported` | `Unavailable` | `InvalidConfiguration` | `InitializationFailed` |
| --- | --- | --- | --- | --- |
| `OnAbsence` | Continue | Continue | Stop | Stop |
| `OnAnyError` | Continue | Continue | Continue | Continue |

`OnAbsence` is the default and is appropriate when fallback means “use another
implementation if this one cannot serve the request or environment.” It stops
on invalid configuration and unexpected initialization failures so that a real
problem is not hidden.

`OnAnyError` is explicitly best effort. Use it only when trying another
provider is correct even after invalid configuration or an unexpected
initialization failure.

Unknown selectors in a chain are not provider errors. They are recorded and
skipped independently of the fallback policy. Named selection still reports an
unknown provider immediately because it contains no next candidate.

## Resolving and Creating Services

Construct a resolver from a registry and policy:

```rust
# use std::sync::Arc;
use qubit_spi::{FallbackPolicy, ProviderRegistry, ProviderResolver};
# struct GreeterConfig { prefix: String }
# trait Greeter: Send + Sync { fn greet(&self, name: &str) -> String; }
# struct GreeterSpec;
# impl qubit_spi::ServiceSpec for GreeterSpec {
#     type Config = GreeterConfig;
#     type Output = Arc<dyn Greeter>;
# }

let registry = ProviderRegistry::<GreeterSpec>::default();
let resolver = ProviderResolver::new(registry, FallbackPolicy::OnAbsence);

assert!(resolver.registry().is_empty());
assert_eq!(FallbackPolicy::OnAbsence, resolver.fallback_policy());
```

Use `create_auto`, `create_named`, or `create_chain` at runtime-input
boundaries. These methods parse raw selectors and convert validation failures
to `ResolutionError`:

```rust,ignore
let service = resolver.create_auto(&config)?;
let service = resolver.create_named(configured_name, &config)?;
let service = resolver.create_chain(configured_chain, &config)?;
```

When the same configured selection is reused, validate it once and call
`create` repeatedly:

```rust,ignore
let selection = ProviderSelection::chain(["cloud", "local"])?;
let first = resolver.create(&selection, &config)?;
let second = resolver.create(&selection, &config)?;
```

This avoids repeatedly allocating and normalizing selectors. Resolver and
registry clones continue to refer to the same immutable catalog.

## Inspecting Successful Results

Resolver methods return `CreatedService<S::Output>`. It retains the canonical
ID of the provider that succeeded, even when selection used an alias:

```rust,ignore
let created = resolver.create_named("builtin", &config)?;
tracing::info!(provider = %created.provider_id(), "created greeter");
let greeting = created.service().greet("Ada");
```

Use `service()` to borrow the output, `into_service()` to discard the provider
identity and take the output, or `into_parts()` to take both owned values.

For direct lookup without fallback, `ProviderRegistry::resolve` returns a
borrowed `ResolvedProvider`. Its `descriptor()` exposes registration metadata
and its `create()` method invokes that one provider. This is useful when code
needs to inspect metadata before creation; ordinary creation flows should use a
resolver so diagnostics and policy handling remain consistent.

## Error Handling and Diagnostics

Errors are separated by lifecycle:

| Error | Meaning |
| --- | --- |
| `ProviderIdError` | A canonical provider ID is empty or noncanonical. |
| `ProviderSelectorError` | Raw selector input normalizes to an empty or invalid token. |
| `ProviderDescriptorError` | An alias is invalid, duplicated, or equal to the canonical ID. |
| `ProviderSelectionError` | A named/chain selector is invalid or a chain is empty. |
| `RegistrationError` | A canonical ID or alias is already claimed in the builder. |
| `ProviderError` | One provider classified a service-creation failure. |
| `ResolutionError` | Selection parsing, lookup, traversal, or creation produced no service. |

`ProviderErrorKind` has four classifications: `Unsupported`, `Unavailable`,
`InvalidConfiguration`, and `InitializationFailed`. Constructors ending in
`_with_source` retain an underlying `Error + Send + Sync + 'static` for the
standard error source chain.

`ResolutionError` distinguishes invalid raw selectors, empty raw chains,
unknown named providers, automatic selection on an empty registry, and an
aggregate `NoProviderSucceeded` outcome. For an aggregate error:

- `attempts()` returns failures in encounter order.
- `terminal_attempt()` returns the last recorded attempt.
- `termination()` reports `Exhausted` or `StoppedByPolicy`.
- `decisive_attempt()` returns the policy-stopping attempt or the only attempt
  in a singleton exhausted result; ambiguous multi-attempt exhaustion returns
  `None`.
- `is_absence()` is true for an unknown named provider or an aggregate made
  only of unknown, unsupported, and unavailable attempts.

Each `AttemptFailure` distinguishes an unknown selector from an invoked
provider error. Provider attempts retain the requested selector when explicit,
the canonical provider ID, the original `ProviderError`, and its source.
`ResolutionError` display output includes ordered attempt diagnostics.

Public error enums are `#[non_exhaustive]`. Match known variants and keep a
wildcard arm:

```rust
use qubit_spi::error::ResolutionError;
use qubit_spi::ResolutionTermination;

fn describe(error: &ResolutionError) -> &'static str {
    match error {
        ResolutionError::InvalidSelector { .. } => "invalid selector",
        ResolutionError::EmptySelection => "empty chain",
        ResolutionError::UnknownProvider { .. } => "unknown provider",
        ResolutionError::EmptyRegistry => "empty registry",
        ResolutionError::NoProviderSucceeded {
            termination: ResolutionTermination::StoppedByPolicy,
            ..
        } => "fallback stopped",
        ResolutionError::NoProviderSucceeded { .. } => "candidates exhausted",
        _ => "future resolution error",
    }
}
```

## Sharing and Performance

Registries are built once and backed by shared immutable storage. Cloning a
registry or resolver increments an `Arc` reference count rather than copying
providers or indexes. Lookup uses a selector index, and automatic candidate
order is computed during `build`, not on each resolution.

Successful `ProviderSelector` parsing allocates owned normalized text. Prefer a
cached `ProviderSelector` or `ProviderSelection` when the same configuration is
used repeatedly. Raw resolver methods are preferable at request or
configuration boundaries because they preserve invalid input and its parsing
source in `ResolutionError`.

Provider factories are `Send + Sync`, and immutable registries/resolvers can be
shared for concurrent lookup and creation. The thread-safety, lifetime, and
allocation behavior of created service outputs remain defined by
`ServiceSpec::Output` and the provider implementation.

## Recommended Practices

- Assemble registries explicitly during startup and fail startup on descriptor
  or registration errors.
- Use stable, lowercase canonical IDs in persisted configuration and reserve
  aliases for compatibility or operator convenience.
- Assign priorities deliberately and remember that canonical ID is the stable
  tie-breaker.
- Use `OnAbsence` unless best-effort continuation after configuration or
  initialization failures is an explicit product requirement.
- Classify `ProviderError` accurately; fallback correctness depends on it.
- Use raw resolver methods at untrusted input boundaries and cache validated
  `ProviderSelection` values for repeated internal calls.
- Record `CreatedService::provider_id()` in logs or metrics so operators can
  identify the selected implementation.
- Inspect ordered attempts and termination instead of parsing display text.
- Include a wildcard arm when matching public error enums.

## Common Problems

**An ID is rejected but a similar selector works.** `ProviderId` requires input
to be canonical already; it does not trim or lowercase. `ProviderSelector`
normalizes configuration input. Store the canonical form as the ID.

**Registration fails after adding an alias.** Canonical IDs and aliases share
one selector namespace. Check the `RegistrationError::DuplicateSelector`
fields to find the current owner and conflicting registration.

**Automatic resolution returns `EmptyRegistry`.** No providers were registered
before `build`, or the wrong typed registry was supplied to the resolver.

**A chain rejects all candidates before resolution.** `ProviderSelection::chain`
and `create_chain` reject an empty chain and stop parsing at the first invalid
selector. Unknown but syntactically valid selectors are different: they become
ordered attempt failures during resolution.

**Fallback stops earlier than expected.** Under `OnAbsence`,
`InvalidConfiguration` and `InitializationFailed` stop traversal. Inspect
`termination()` and `decisive_attempt()` to identify the policy-stopping
provider error.

**`decisive_attempt()` returns `None`.** Multiple candidates were exhausted and
no single failure explains the aggregate outcome. Inspect every entry returned
by `attempts()`.

## Complete Example

This example registers a preferred cloud provider and a local fallback. The
cloud provider reports that it is unavailable, so automatic resolution under
`OnAbsence` continues to the local provider.

```rust
use std::sync::Arc;

use qubit_spi::error::{ProviderError, ResolutionError};
use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderResolver,
    ResolutionTermination,
    ServiceProvider,
    ServiceSpec,
};

trait Greeter: Send + Sync {
    fn greet(&self, name: &str) -> String;
}

struct GreeterConfig {
    prefix: String,
    cloud_available: bool,
}

struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = GreeterConfig;
    type Output = Arc<dyn Greeter>;
}

struct TextGreeter {
    prefix: String,
}

impl Greeter for TextGreeter {
    fn greet(&self, name: &str) -> String {
        format!("{} {name}", self.prefix)
    }
}

struct CloudProvider;

impl ServiceProvider<GreeterSpec> for CloudProvider {
    fn create(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderError> {
        if !config.cloud_available {
            return Err(ProviderError::unavailable(
                "the cloud greeting service is offline",
            ));
        }
        Ok(Arc::new(TextGreeter {
            prefix: format!("{} from the cloud,", config.prefix),
        }))
    }
}

struct LocalProvider;

impl ServiceProvider<GreeterSpec> for LocalProvider {
    fn create(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderError> {
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "the greeting prefix must not be empty",
            ));
        }
        Ok(Arc::new(TextGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<GreeterSpec>::builder();
    builder.register(
        ProviderDescriptor::new(ProviderId::new("cloud")?)
            .with_aliases(["remote"])?
            .with_priority(100),
        CloudProvider,
    )?;
    builder.register(
        ProviderDescriptor::new(ProviderId::new("local")?)
            .with_aliases(["builtin"])?
            .with_priority(10),
        LocalProvider,
    )?;

    let resolver = ProviderResolver::new(
        builder.build(),
        FallbackPolicy::OnAbsence,
    );
    let config = GreeterConfig {
        prefix: "Hello,".to_owned(),
        cloud_available: false,
    };

    match resolver.create_auto(&config) {
        Ok(created) => {
            assert_eq!("local", created.provider_id().as_str());
            assert_eq!("Hello, Ada", created.service().greet("Ada"));
        }
        Err(error) => report_resolution_error(&error),
    }

    let named = resolver.create_named("builtin", &config)?;
    assert_eq!("local", named.provider_id().as_str());
    Ok(())
}

fn report_resolution_error(error: &ResolutionError) {
    match error.termination() {
        Some(ResolutionTermination::StoppedByPolicy) => {
            eprintln!("resolution stopped: {error}");
        }
        Some(ResolutionTermination::Exhausted) => {
            eprintln!("all candidates failed: {error}");
        }
        None => eprintln!("selection failed: {error}"),
        _ => eprintln!("resolution failed: {error}"),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("startup failed: {error}");
    }
}
```

## API Reference

The complete generated API documentation is available on
[docs.rs](https://docs.rs/qubit-spi). The principal entry points are:

| Area | Types |
| --- | --- |
| Service contract | `ServiceSpec`, `ServiceProvider` |
| Identity and metadata | `ProviderId`, `ProviderSelector`, `ProviderDescriptor` |
| Registry | `ProviderRegistryBuilder`, `ProviderRegistry`, `ResolvedProvider` |
| Selection and resolution | `ProviderSelection`, `ProviderSelectionKind`, `FallbackPolicy`, `ProviderResolver` |
| Results | `CreatedService`, `ResolutionTermination` |
| Errors | `ProviderIdError`, `ProviderSelectorError`, `ProviderDescriptorError`, `ProviderSelectionError`, `RegistrationError`, `ProviderError`, `ProviderErrorKind`, `AttemptFailure`, `ResolutionError` |
