# Qubit SPI User Guide

This guide explains Qubit SPI's runtime provider model. It covers the complete
lifecycle from App startup registration to downstream service use, including
selection, configuration, fallback, diagnostics, threading, and global
facades.

## The Problem Qubit SPI Solves

Suppose a reusable `lib-foo` library needs a Greeter. `lib-foo` should not
choose or construct a concrete implementation itself because the final App may
need a provider supplied by its deployment.

The intended runtime relationship is:

1. The App registers available providers during startup.
2. The App may set a process-wide default provider selection.
3. `lib-foo` later resolves either its own explicit selection or that default.
4. The resolved provider creates a service with explicit or default config.
5. `lib-foo` uses the returned service without knowing its concrete type.

This is a service-provider registry, not general dependency injection. It
standardizes how implementations are registered, selected, and created while
keeping the service's business API in its own domain crate.

## First Principles: Three Separate Stages

The central design rule is that registration, selection, and creation answer
different questions and must not be collapsed into one operation.

### Registration: What Exists?

Registration installs a provider with `ProviderMetadata` plus the matching
sync or async creation capability. Sync and async providers live in separate
registries. Registry state contains identity and lookup metadata, not a
created service.

Registration can fail because a canonical ID or alias is already owned. It
does not parse a request selection and does not create a service.

### Selection: What May Be Tried?

A `ProviderSelection` identifies a named provider, a caller-ordered chain, or
the Registry's automatic order. `ProviderRegistry::resolve_selected`
translates an explicit selection into a point-in-time candidate snapshot;
`ProviderRegistry::resolve` does the same for the Registry default. Both
return `ResolvingServiceProvider<S>`.

Selection does not require `S::Config` and does not invoke provider code. It
can fail because the requested provider or candidate set does not exist.

### Creation: Can a Candidate Build the Service?

`ResolvingServiceProvider<S>` is a composing resolver with inherent `create`
methods. It invokes candidates with `S::Config`, applies the fallback policy
stored in the selection, and returns `S::Output` directly on success. Its
aggregate failures are reported as `ProviderCreationError`, so it is not a
`ServiceProvider<S>` implementation. The async resolver has the same behavior
but awaits each async provider invocation.

Creation can fail because a provider does not support the request, is
unavailable, rejects the config, or cannot initialize. Aggregate errors retain
only the providers that were actually called.

```text
metadata + provider --register--> sync or async Registry
                                      │
ProviderSelection ---------------- resolve
                                      │
                                      ▼
                         ResolvingServiceProvider
                                      │
S::Config ------------------------- create
                                      │
                                      ▼
                                 S::Output
```

## Core Types

| Type | Responsibility |
| --- | --- |
| `ServiceSpec` | Binds one service family's `Config` type |
| `SyncServiceSpec` / `AsyncServiceSpec` | Bind independent sync and async output types |
| `ServiceProvider<S>` / `AsyncServiceProvider<S>` | Create the corresponding output from `S::Config` |
| `ProviderDefinition<S>` / `AsyncProviderDefinition<S>` | Marker traits combining metadata with the matching sync or async creation contract |
| `ProviderMetadata` | Supplies the provider-owned descriptor |
| `ProviderFuture<'a, T>` | Runtime-independent boxed, sendable future returned by `AsyncServiceProvider` implementations |
| `ProviderId` | Stable canonical identity: nonempty lowercase ASCII, alphanumeric endpoints, separators `-`/`_`/`.`/`+` only; never normalized |
| `ProviderDescriptor` | Stores canonical ID, aliases, and automatic priority |
| `ProviderRegistry<S>` / `AsyncProviderRegistry<S>` | Own independent sync or async runtime registration and default-selection state |
| `ProviderSelection` | Describes candidates and the creation fallback policy |
| `ResolvingServiceProvider<S>` / `AsyncResolvingServiceProvider<S>` | Own the corresponding resolved candidate snapshot and create sync or async output |

The generic `S` keeps providers for unrelated service families from being
mixed. A MIME provider cannot be registered in a filesystem Registry because
their `ServiceSpec` types differ.

## Defining a Service Family

Start with the business capability. It should contain operations consumers call
after initialization. Construction settings belong in a separate config type.

```rust
use std::{
    error::Error,
    fmt,
    sync::Arc,
};

use qubit_spi::{ServiceSpec, SyncServiceSpec};

/// Business interface implemented by every Greeter service.
trait Greeter: Send + Sync {
    fn greet(&self, name: &str) -> String;
}

/// Configuration passed to a provider when it creates a Greeter.
#[derive(Clone)]
struct GreeterConfig {
    /// Text placed before the name in each greeting.
    prefix: String,
}

impl Default for GreeterConfig {
    fn default() -> Self {
        Self {
            prefix: "Hello".to_owned(),
        }
    }
}

/// Domain error retained when a Greeter provider fails.
#[derive(Debug)]
struct GreeterError {
    message: String,
}

impl GreeterError {
    fn invalid_configuration(message: &str) -> Self {
        Self {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for GreeterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for GreeterError {}

/// Connects the Greeter configuration and output types to Qubit SPI.
struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    // Input accepted by Greeter providers during service creation.
    type Config = GreeterConfig;
    // Domain error preserved by classified provider failures.
    type Error = GreeterError;
}

impl SyncServiceSpec for GreeterSpec {
    // Service object returned to consumers after successful creation.
    type Output = Arc<dyn Greeter>;
}
```

`SyncServiceSpec::Output` or `AsyncServiceSpec::Output` is the complete value consumers need. Common choices are
`Arc<dyn Trait>`, a concrete client, or a lightweight handle. Qubit SPI does not
wrap successful outputs with provider metadata and does not cache them.

`ServiceSpec::Config` may be unsized. `create()` is available only when
that config implements `Default`; `create_configured(&config)` is always available.

## Implementing a Self-Described Provider

A registrable Provider implements two contracts:

1. `ServiceProvider<S>` for creation behavior.
2. `ProviderMetadata` for stable registration metadata.

The canonical ID passed to `ProviderId::new` must already be a nonempty
lowercase ASCII token with alphanumeric endpoints and only the separators
`-`, `_`, `.`, and `+`; it is never trimmed or lowercased.

```rust
use std::sync::Arc;

use qubit_spi::error::ProviderFailure;
use qubit_spi::{
    ProviderDescriptor, ProviderId, ProviderMetadata, ServiceProvider,
};

/// Concrete Greeter created by the friendly provider.
struct FriendlyGreeter {
    /// Greeting prefix copied from the creation configuration.
    prefix: String,
}

impl Greeter for FriendlyGreeter {
    fn greet(&self, name: &str) -> String {
        format!("{}, {}!", self.prefix, name)
    }
}

/// Self-described provider exported for Apps to register explicitly.
pub struct FriendlyGreeterProvider;

impl ServiceProvider<GreeterSpec> for FriendlyGreeterProvider {
    fn create_configured(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderFailure<GreeterError>> {
        if config.prefix.trim().is_empty() {
            return Err(ProviderFailure::invalid_configuration(
                GreeterError::invalid_configuration(
                    "the greeting prefix must not be empty",
                ),
            ));
        }
        Ok(Arc::new(FriendlyGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}

impl ProviderMetadata for FriendlyGreeterProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(
            ProviderId::new("friendly").expect("static provider ID is valid"),
        )
        .with_aliases(["default-greeter", "friendly-greeter"])
        .expect("static aliases are valid")
        .with_priority(100)
    }
}
```

## Asynchronous Providers and Registries

An async-capable service family additionally implements `AsyncServiceSpec`.
That trait requires `Config: Sync` and defines an independent
`Output: Send + 'static`. An `AsyncServiceProvider<S>` returns the sendable,
boxed `ProviderFuture`, which keeps Qubit SPI independent of Tokio, async-std,
or another executor. `ProviderMetadata` supplies registration identity;
together those two traits automatically satisfy the
`AsyncProviderDefinition<S>` marker accepted by `AsyncProviderRegistry<S>`.
No explicit marker implementation is needed. Resolution produces an
`AsyncResolvingServiceProvider<S>` that owns the candidate snapshot and awaits
providers according to the selected fallback policy.

The sync and async registries reuse the same `ProviderSelection`,
`MissingProviderPolicy`, and `FallbackPolicy` types. Resolution failures from
either Registry are `ProviderResolutionError`, and both resolver types
aggregate creation failures as `ProviderCreationError`. Sync resolver creation
returns the output directly, while async resolver creation is `async` and must
be awaited to obtain the output. Calling an async leaf provider directly
returns a `ProviderFuture`.

```rust,ignore
use std::sync::Arc;

use qubit_spi::error::ProviderFailure;
use qubit_spi::{
    AsyncProviderRegistry, AsyncServiceProvider, AsyncServiceSpec,
    ProviderDescriptor, ProviderFuture, ProviderId, ProviderMetadata,
    ProviderSelection,
};

impl AsyncServiceSpec for GreeterSpec {
    type Output = Arc<dyn Greeter>;
}

/// Async Greeter provider registered by the App.
pub struct AsyncFriendlyGreeterProvider;

impl AsyncServiceProvider<GreeterSpec> for AsyncFriendlyGreeterProvider {
    fn create_configured<'a>(
        &'a self,
        config: &'a GreeterConfig,
    ) -> ProviderFuture<'a, Result<Arc<dyn Greeter>, ProviderFailure<GreeterError>>> {
        Box::pin(async move {
            if config.prefix.trim().is_empty() {
                return Err(ProviderFailure::invalid_configuration(
                    GreeterError::invalid_configuration(
                        "the greeting prefix must not be empty",
                    ),
                ));
            }
            Ok(Arc::new(FriendlyGreeter {
                prefix: config.prefix.clone(),
            }) as Arc<dyn Greeter>)
        })
    }
}

impl ProviderMetadata for AsyncFriendlyGreeterProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(
            ProviderId::new("async-friendly")
                .expect("static provider ID is valid"),
        )
        .with_aliases(["async-default-greeter"])
        .expect("static aliases are valid")
        .with_priority(100)
    }
}

let registry = AsyncProviderRegistry::<GreeterSpec>::default();
registry.register(AsyncFriendlyGreeterProvider)?;
let selection = ProviderSelection::named("async-friendly")?;
let resolver = registry.resolve_selected(&selection)?;
let greeter = resolver.create_configured(&config).await?;
```

Registration, queries, default-selection changes, and resolution are all
synchronous because they only manipulate in-memory metadata and snapshots.
Only provider creation is asynchronous. No Registry lock is retained while a
returned future is polled, so pending creation does not block registration or
queries. The resolver's candidate snapshot does not observe providers
registered after resolution.

The default async `create()` method is available only when
`S::Config: Default + Send`; `create_configured(&config)` needs the service
family's baseline `Config: Sync` constraint instead. Sync and async registries
do not share registration state. A provider implementation that supports both
creation modes must be registered separately in each Registry.

### Why the Descriptor Belongs to the Provider

The Provider's identity and its creation implementation are one registration
unit. Requiring callers to pass them separately allows mismatched metadata and
makes third-party installation unnecessarily error-prone. A self-described
Provider lets the App write:

```rust,ignore
registry.register(FriendlyGreeterProvider)?;
```

Registration calls `descriptor()` before acquiring the Registry write lock,
then stores a descriptor snapshot. Later Provider state changes cannot mutate
registered ID, aliases, or priority.

### Canonical IDs, Aliases, and Priority

`ProviderId` must already be canonical; construction never trims whitespace or
changes letter case. A legal ID is a nonempty lowercase ASCII token whose first
and last characters are letters or digits (`a`–`z`, `0`–`9`) and whose remaining
characters are alphanumeric or one of the separators `-`, `_`, `.`, and `+`.
Surrounding whitespace, uppercase letters, and other punctuation are rejected.

`ProviderSelector` is an input-boundary type. Parsing trims whitespace,
ASCII-lowercases the value, and validates the same token grammar. Therefore a
configured value such as `" Friendly-Greeter "` resolves the normalized alias
`friendly-greeter`.

Aliases must not duplicate the canonical ID or one another after normalization.
Priority affects only automatic selection. Higher values come first; equal
priorities are ordered by canonical ID ascending.

## Creating and Sharing a Runtime Registry

The simplest Registry is empty and runtime mutable:

```rust,ignore
let registry = ProviderRegistry::<GreeterSpec>::default();
registry.register(FriendlyGreeterProvider)?;
```

An isolated Registry is assembled directly through its runtime registration
API:

```rust,ignore
let registry = ProviderRegistry::<GreeterSpec>::default();
registry.register(FriendlyGreeterProvider)?;
registry.register(AnotherProvider)?;
```

`ProviderRegistry<S>` and `AsyncProviderRegistry<S>` expose parallel synchronous
catalog APIs, but their registration and default-selection states are
independent. A provider supporting both creation modes must be registered in
both. Use `register_shared` when the Provider is already stored in the matching
`Arc<dyn ProviderDefinition<S>>` or
`Arc<dyn AsyncProviderDefinition<S>>`; otherwise prefer `register(provider)`.

### Clone and Synchronization Semantics

Registry clones share one `Arc<RwLock<...>>` state:

```rust,ignore
let library_registry = registry.clone();
registry.register(FriendlyGreeterProvider)?;
assert_eq!(1, library_registry.len());
```

Registration and default-selection updates become visible through every clone.
Methods returning descriptors, IDs, defaults, or resolved providers return
owned snapshots. Registry locks are not held while third-party Provider code
runs.

Registration is atomic with respect to selector conflicts. If an ID or alias
is already owned, the Registry remains unchanged and returns
`RegistrationError::DuplicateSelector` naming both Provider IDs.

## The Complete Three-Library and App Pattern

Qubit SPI deliberately does not define one universal global Registry: each
service family has a different `ServiceSpec`. The domain crate that owns the
service trait exposes the appropriate singleton. This complete example splits
the four responsibilities across three independently published libraries and
one App.

Cargo package names use hyphens below; Rust refers to those crates with
underscores. The `Cargo.toml` files are omitted for brevity.

### 1. `lib-greeter`: Define the Service and Global Registry

`lib-greeter` owns the service contract and the one Registry instance shared
by consumers, providers, and the final App.

```rust
// lib-greeter/src/lib.rs
use std::{
    error::Error,
    fmt,
    sync::{Arc, LazyLock},
};

use qubit_spi::{ProviderRegistry, ServiceSpec, SyncServiceSpec};

/// Business interface implemented by every Greeter service.
pub trait Greeter: Send + Sync {
    fn greet(&self, name: &str) -> String;
}

/// Configuration passed to a provider when it creates a Greeter.
#[derive(Clone)]
pub struct GreeterConfig {
    /// Text placed before the name in each greeting.
    pub prefix: String,
}

impl Default for GreeterConfig {
    fn default() -> Self {
        Self {
            prefix: "Hello".to_owned(),
        }
    }
}

/// Domain error retained when a Greeter provider fails.
#[derive(Debug)]
pub struct GreeterError;

impl fmt::Display for GreeterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("greeter provider failed")
    }
}

impl Error for GreeterError {}

/// Connects the Greeter configuration and output types to Qubit SPI.
pub struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    // Input accepted by Greeter providers during service creation.
    type Config = GreeterConfig;
    // Domain error preserved by classified provider failures.
    type Error = GreeterError;
}

impl SyncServiceSpec for GreeterSpec {
    // Service object returned to consumers after successful creation.
    type Output = Arc<dyn Greeter>;
}

/// Process-wide Greeter provider registry shared by the App and all libraries.
pub static GREETER_REGISTRY: LazyLock<ProviderRegistry<GreeterSpec>> =
    LazyLock::new(ProviderRegistry::default);
```

### 2. `lib-foo`: Consume the Default Service

`lib-foo` depends on `lib-greeter` and `qubit-spi`, but not on any concrete
Greeter implementation.

```rust
// lib-foo/src/lib.rs
use lib_greeter::GREETER_REGISTRY;

/// Creates the App-selected default Greeter and prints one greeting.
pub fn foo() -> Result<(), Box<dyn std::error::Error>> {
    let provider = GREETER_REGISTRY.resolve()?;
    let greeter = provider.create()?;
    println!("{}", greeter.greet("Rust"));
    Ok(())
}
```

### 3. `lib-friendly-greeter`: Supply a Third-Party Provider

`lib-friendly-greeter` depends on `lib-greeter` and `qubit-spi`. It implements
the Greeter contract and publishes a self-described provider, but it does not
modify global state by registering itself.

```rust
// lib-friendly-greeter/src/lib.rs
use std::sync::Arc;

use lib_greeter::{Greeter, GreeterConfig, GreeterError, GreeterSpec};
use qubit_spi::error::ProviderFailure;
use qubit_spi::{
    ProviderDescriptor, ProviderId, ProviderMetadata, ServiceProvider,
};

/// Concrete Greeter created by the friendly provider.
struct FriendlyGreeter {
    /// Greeting prefix copied from the creation configuration.
    prefix: String,
}

impl Greeter for FriendlyGreeter {
    fn greet(&self, name: &str) -> String {
        format!("{}, {}!", self.prefix, name)
    }
}

/// Self-described provider exported for Apps to register explicitly.
pub struct FriendlyGreeterProvider;

impl ServiceProvider<GreeterSpec> for FriendlyGreeterProvider {
    fn create_configured(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderFailure<GreeterError>> {
        Ok(Arc::new(FriendlyGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}

impl ProviderMetadata for FriendlyGreeterProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(
            ProviderId::new("friendly").expect("static provider ID is valid"),
        )
        .with_priority(100)
    }
}
```

### 4. `app.rs`: Register the Provider and Run `lib-foo`

The App depends on all three libraries and owns the composition policy. It
registers the third-party provider before any downstream code requests a
Greeter, sets the process default, and then calls `foo()`.

```rust
// app.rs
use lib_foo::foo;
use lib_friendly_greeter::FriendlyGreeterProvider;
use lib_greeter::GREETER_REGISTRY;
use qubit_spi::ProviderSelection;

// Application composition root: install a provider before calling lib-foo.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    GREETER_REGISTRY.register(FriendlyGreeterProvider)?;
    GREETER_REGISTRY
        .set_default_selection(ProviderSelection::named("friendly")?);
    foo()
}
```

The program prints `Hello, Rust!`. The App and `lib-foo` coordinate through the
same `GREETER_REGISTRY` from `lib-greeter`; neither `lib-foo` nor
`lib-greeter` depends on `lib-friendly-greeter`.

Startup ordering matters: configure the global Registry before downstream code
first requests the service. A `ResolvingServiceProvider` already obtained by a
consumer remains a point-in-time snapshot; later registrations affect future
resolutions, not that existing snapshot.

Cargo normally unifies compatible versions of `lib-greeter`. If incompatible
versions are linked simultaneously, each crate version owns a separate static
Registry. The App and `lib-foo` must use the same linked `lib-greeter` instance
to share the singleton.

## Selecting Providers

Selection is a value object. It can come from a config file, command-line input,
hard-coded library requirements, or an App default. It is not required to live
inside the service's config type.

### Named Selection

```rust,ignore
let selection = ProviderSelection::named("friendly")?;
let provider = registry.resolve_selected(&selection)?;
```

Named selection resolves exactly one canonical ID or alias. An unknown selector
returns `ProviderResolutionError::UnknownProviders`. Because it contains one
candidate, its fallback policy never causes another Provider to run.

### Ordered Chain

```rust,ignore
let selection = ProviderSelection::chain([
    "remote-greeter",
    "friendly",
    "minimal",
])?;
let provider = registry.resolve_selected(&selection)?;
```

Chain order is caller order. `chain()` is strict and rejects the entire
selection if any selector is unknown. Use `chain_allowing_missing()` only when
uninstalled optional plugins should be skipped. If multiple selectors refer to
the same Provider through its ID and aliases, that Provider appears once at its
first position. A lenient chain fails with `NoCandidates` when no entry matches.

### Automatic Selection

```rust,ignore
let provider = registry.resolve_selected(&ProviderSelection::auto())?;
```

Automatic selection includes every registered Provider in deterministic order:

1. priority descending;
2. canonical ID ascending for equal priority.

An empty Registry returns `ProviderResolutionError::EmptyRegistry`.

### Registry Default Selection

A new Registry defaults to `ProviderSelection::auto()` with
`FallbackPolicy::OnAbsence`. The App may replace that value at runtime:

```rust,ignore
let default = ProviderSelection::chain(["remote", "friendly"])?
    .with_fallback_policy(FallbackPolicy::OnAbsence);
registry.set_default_selection(default);

let snapshot = registry.default_selection();
let provider = registry.resolve()?;
```

`set_default_selection` stores a validated selection but does not require its
providers to exist yet. This permits setting policy before registration.
`resolve_default_snapshot()` atomically captures the current default
`ProviderSelection` and its candidates from one Registry catalog snapshot. On
success, the returned `ResolvingServiceProvider<S>` owns both the captured
selection policy and candidate handles. The asynchronous Registry provides the
same operation and returns an `AsyncResolvingServiceProvider<S>`; catalog
resolution itself is synchronous.

Call `resolve_default_snapshot_with_selection()` when the caller must validate
input against the captured default selection before creating from the same
candidate snapshot. It returns the owned `ProviderSelection` and resolver as a
pair; its asynchronous counterpart has the same contract.

If resolution fails, the returned `ProviderResolutionError` is also detached
from the Registry. Selector-based failures retain the selectors captured from
that selection through `error.selectors()`; `EmptyRegistry` has no selectors.
Registering a matching Provider later does not change the existing error into a
success. Likewise, a successful resolver does not see Providers registered
after the snapshot. Call `resolve_default_snapshot()` again to obtain a newer
selection and candidate snapshot. `resolve()` remains a compatibility alias
with the same behavior.

### Selection and Config Are Independent

These are all valid:

```rust,ignore
// Registry default selection and default config.
let service = registry.resolve()?.create()?;

// Explicit selection and default config.
let service = registry.resolve_selected(&selection)?.create()?;

// Registry default selection and explicit config.
let service = registry.resolve()?.create_configured(&config)?;

// Explicit selection and explicit config.
let service = registry.resolve_selected(&selection)?.create_configured(&config)?;
```

Do not force a Provider selection field into every service config. A config may
offer a selection as one convenience source, but Registry APIs remain usable by
callers that have no config object.

## Creating the Service

`ProviderRegistry::resolve_default_snapshot()` and
`ProviderRegistry::resolve_selected` return
`ResolvingServiceProvider<S>`. This type is a composing resolver: it owns
candidate handles and applies the selection's fallback policy when `create` is
called. Its inherent creation methods return aggregate
`ProviderCreationError` values rather than the leaf `ProviderFailure<E>` required by
`ServiceProvider<S>`.

`ProviderRegistry::resolve()` is a compatibility alias for
`resolve_default_snapshot()`. Both APIs capture the default selection and
candidates together, so later Registry changes do not affect a resolver or a
resolution error that has already been returned.

`ProviderRegistry::resolve_default_snapshot_with_selection()` returns the
captured `ProviderSelection` alongside the resolver when the caller needs to
compare that selection with configuration. `AsyncProviderRegistry` exposes the
same method and snapshot contract.

The corresponding `AsyncProviderRegistry` methods return
`AsyncResolvingServiceProvider<S>`. Its inherent creation methods are async;
await them to obtain the async `S::Output`. The leaf
`AsyncServiceProvider<S>` contract returns the boxed `ProviderFuture` type.

```rust,ignore
let provider = registry.resolve_selected(&selection)?;
let service = provider.create_configured(&config)?;

let async_provider = async_registry.resolve_selected(&selection)?;
let async_service = async_provider.create_configured(&config).await?;
```

Sync default `create()` requires `S::Config: Default`. Async default `create()`
requires `S::Config: Default + Send`; both modes can always use an explicit
config when the service specification's own bounds are satisfied.

Successful creation returns `S::Output` directly. The public API does not
expose successful attempt data.

Qubit SPI creates a new output for every `create` call. Cache or clone the
returned handle in App or library code when construction is expensive.

## Fallback Policy

Fallback belongs to `ProviderSelection` because it is part of the caller's
request policy, not permanent Registry state and not service configuration.

| Policy | Continue after `Unsupported` | Continue after `Unavailable` | Continue after invalid config or initialization failure |
| --- | --- | --- | --- |
| `Never` | No | No | No |
| `OnAbsence` | Yes | Yes | No |
| `OnAnyError` | Yes | Yes | Yes |

`OnAbsence` is the default and safest general policy. It treats missing
capability or environment as reasons to try an alternative, while stopping on
likely programming or deployment errors. Use `OnAnyError` only when degraded
best-effort behavior is explicitly desired.

Fallback is evaluated after a Provider returns a leaf `ProviderFailure<E>`.
Only a sync or async resolver aggregates attempts into
`ProviderCreationError`.

## Error Model

Errors follow the three lifecycle stages plus input validation.

### Definition and Registration Errors

- `ProviderIdError`: a canonical ID is empty or violates the lowercase ASCII
  token grammar (alphanumeric endpoints; separators `-`, `_`, `.`, `+` only).
- `ProviderSelectorError`: normalized user/config input is empty or invalid.
- `ProviderDescriptorError`: an alias is invalid, duplicated, or matches ID.
- `RegistrationError`: an ID or alias conflicts with Registry state.

### Selection Construction Errors

`ProviderSelectionBuildError` is returned while constructing a validated
selection:

- `InvalidSelector`: raw selection input is invalid;
- `EmptyChain`: caller supplied no chain entries.

### Provider Resolution Errors

`ProviderResolutionError` is returned before any Provider is invoked:

- `UnknownProviders`: named or strict-chain selection contains unknown entries;
- `NoCandidates`: no entry in a nonempty chain matched;
- `EmptyRegistry`: automatic selection has no Provider.

These errors contain no Provider creation attempts because none occurred.

### Leaf Provider Errors

One concrete Provider reports a `ProviderFailure<E>` classified by
`ProviderFailureKind`:

- `Unsupported`: provider cannot serve this request;
- `Unavailable`: provider or required environment is absent;
- `InvalidConfiguration`: provider rejects the supplied config;
- `InitializationFailed`: provider failed unexpectedly while constructing.

Use the `_with_source` constructors to retain an underlying error. Registry
internals do not log or collect external observations; consumers receive a
complete error chain when an operation fails.

### Aggregate Creation Errors

`ProviderCreationError` is always a nonempty aggregate produced by a resolver.

Every `ProviderAttemptFailure` contains the canonical ID and original
`ProviderFailure<E>` of an actually invoked Provider. Missing chain selectors do not
fabricate attempts.

`ProviderCreationTermination` explains traversal:

- `Exhausted`: every admitted candidate was tried;
- `StoppedByPolicy`: fallback rejected continuing after the terminal failure.

Useful queries include:

```rust,ignore
if error.is_absence() {
    // Every relevant failure is Unsupported or Unavailable.
}

for attempt in error.attempts() {
    eprintln!("{}: {}", attempt.provider_id(), attempt.error());
}

match error.termination() {
    ProviderCreationTermination::Exhausted => { /* ... */ }
    ProviderCreationTermination::StoppedByPolicy => { /* ... */ }
    _ => { /* future non-exhaustive variant */ }
}
```

`decisive_attempt()` always returns the final actual attempt, which directly
stopped traversal or exhausted the candidate snapshot.

## Concurrency and Snapshot Semantics

`ProviderRegistry<S>` and `AsyncProviderRegistry<S>` are `Send + Sync` when
their stored Provider definitions are, which the Provider traits require.
Each Registry has its own shared `RwLock` state.

- Registration obtains a write lock only after calling `descriptor()`.
- Default-selection replacement obtains a short write lock.
- Resolution obtains a read lock while copying candidate handles.
- Sync Provider creation and async future polling occur after the lock is
  released; a pending future therefore does not block registration or queries.
- `parking_lot::RwLock` does not poison; a panic releases the lock normally.

A sync or async resolved provider owns `Arc` handles to its candidates, so it
remains usable after its Registry is cloned, changed, or dropped. Neither
snapshot sees later registrations. Resolve again to obtain new candidates.

## Recommended Practices

1. Define one `ServiceSpec` per independently selectable service family.
2. Let the domain crate own the service trait and optional global facade.
3. Make each registrable Provider implement `ProviderMetadata` directly.
4. Choose `ProviderRegistry` or `AsyncProviderRegistry` to match the creation
   mode; register a dual-mode implementation separately in both.
5. Register App-specific providers before downstream service use begins.
6. Store default policy in the Registry; pass explicit selection only when the
   caller has a real requirement.
7. Keep selection independent from service configuration.
8. Prefer `OnAbsence`; justify `OnAnyError` at the call site.
9. Return classified `ProviderFailure<E>` values with causal sources.
10. Cache expensive service outputs outside Qubit SPI.
11. Use isolated registries in tests that mutate registrations or defaults.

## Troubleshooting

### A registered provider cannot be found

Check the canonical ID and normalized aliases returned by `descriptor()`. Use
`registry.provider_ids()` and `registry.descriptors()` to inspect snapshots.
Remember that `ProviderId` is never normalized and must already satisfy the
canonical token rules, while `ProviderSelector` trims and lowercases input.

### `resolve()` chooses an unexpected provider

Inspect `registry.default_selection()`. A new Registry defaults to automatic
selection, which uses priority descending and canonical ID ascending. If App
startup should pick one provider, call `set_default_selection` explicitly.

### Fallback does not continue

Inspect the terminal attempt's `ProviderFailureKind` and the selection's policy.
`OnAbsence` stops on `InvalidConfiguration` and `InitializationFailed` by
design. Named selection has no second candidate.

### A newly registered provider is not visible

Existing Registry clones see new registrations, but an already resolved
`ResolvingServiceProvider` is a snapshot. Resolve again. For global facades,
also confirm App and library link the same domain-crate version.

### `create()` is unavailable

Sync `create()` requires `S::Config: Default`; async `create()` requires
`S::Config: Default + Send`. Otherwise construct a config and call
`create_configured(&config)` (and `.await` its async future).

### Duplicate registration fails during repeated tests

Process-wide registries intentionally retain state. Prefer an isolated
`ProviderRegistry::default()` per test, or run the global mutation scenario in
an isolated process.

## API Reference

| API | Purpose |
| --- | --- |
| `ServiceSpec` | Bind the config type |
| `SyncServiceSpec` / `AsyncServiceSpec` | Bind sync and async output types |
| `ServiceProvider::create_configured` | Create with explicit config |
| `ServiceProvider::create` | Create with `Config::default()` |
| `AsyncServiceProvider::create_configured` | Create asynchronously with explicit config |
| `AsyncServiceProvider::create` | Create asynchronously with `Config::default()` when `Config: Default + Send` |
| `ProviderFuture` | Runtime-independent boxed, sendable future returned by `AsyncServiceProvider` implementations |
| `ProviderMetadata::descriptor` | Self-describe a registrable Provider |
| `ProviderDefinition` / `AsyncProviderDefinition` | Marker traits combining metadata with sync or async creation |
| `ProviderRegistry::register` | Register an owned Provider at runtime |
| `ProviderRegistry::register_shared` | Register an existing shared Provider |
| `ProviderRegistry::default_selection` | Read an owned snapshot of the current default selection |
| `ProviderRegistry::set_default_selection` | Replace the process/component default policy |
| `ProviderRegistry::resolve_selected` | Resolve an explicit selection |
| `ProviderRegistry::resolve_default_snapshot` | Atomically resolve the current default selection and candidate snapshot |
| `ProviderRegistry::resolve_default_snapshot_with_selection` | Return the captured default selection and candidate snapshot together |
| `ProviderRegistry::resolve` | Compatibility alias for `resolve_default_snapshot` |
| `ProviderRegistry::descriptors` | Snapshot registration metadata |
| `ProviderRegistry::provider_ids` | Snapshot canonical IDs |
| `AsyncProviderRegistry::register` / `register_shared` | Register an owned or shared async Provider synchronously |
| `AsyncProviderRegistry::set_default_selection` / `default_selection` | Replace or snapshot the async Registry default policy |
| `AsyncProviderRegistry::resolve_selected` | Resolve an explicit selection synchronously |
| `AsyncProviderRegistry::resolve_default_snapshot` | Atomically resolve the async Registry's current default selection and candidate snapshot |
| `AsyncProviderRegistry::resolve_default_snapshot_with_selection` | Return the captured async default selection and candidate snapshot together |
| `AsyncProviderRegistry::resolve` | Compatibility alias for `resolve_default_snapshot` |
| `AsyncProviderRegistry::descriptors` / `provider_ids` | Snapshot async registration metadata or canonical IDs |
| `AsyncProviderRegistry::len` / `is_empty` | Query the async Registry size or emptiness |
| `ProviderSelection::named` | Select exactly one ID or alias |
| `ProviderSelection::chain` | Strictly select caller-ordered candidates |
| `ProviderSelection::chain_allowing_missing` | Explicitly ignore unregistered chain entries |
| `ProviderSelection::auto` | Select all providers deterministically |
| `ProviderSelection::with_fallback_policy` | Attach creation fallback policy |
| `ResolvingServiceProvider` | Create through a resolved candidate snapshot |
| `AsyncResolvingServiceProvider` | Return futures that create through an async candidate snapshot |

For exact signatures and non-exhaustive error variants, use the
[generated API documentation](https://docs.rs/qubit-spi).
