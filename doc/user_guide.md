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

Registration installs a `ProviderDefinition<S>` in a `ProviderRegistry<S>`.
The Provider supplies both creation behavior and its own descriptor. Registry
state contains provider identity and lookup metadata, not a created service.

Registration can fail because a canonical ID or alias is already owned. It
does not parse a request selection and does not create a service.

### Selection: What May Be Tried?

A `ProviderSelection` identifies a named provider, a caller-ordered chain, or
the Registry's automatic order. `ProviderRegistry::resolve` translates that
selection into a point-in-time candidate snapshot represented by
`ResolvingServiceProvider<S>`.

Selection does not require `S::Config` and does not invoke provider code. It
can fail because the requested provider or candidate set does not exist.

### Creation: Can a Candidate Build the Service?

`ResolvingServiceProvider<S>` implements `ServiceProvider<S>`. Its `create`
method invokes candidates with `S::Config`, applies the fallback policy stored
in the selection, and returns `S::Output` directly on success.

Creation can fail because a provider does not support the request, is
unavailable, rejects the config, or cannot initialize. Aggregate errors retain
only the providers that were actually called.

```text
ProviderDefinition --register--> ProviderRegistry
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
| `ServiceSpec` | Binds one service family's `Config` and `Output` types |
| `ServiceProvider<S>` | Creates `S::Output` from `S::Config` |
| `ProviderDefinition<S>` | Adds a self-owned descriptor to a service provider |
| `ProviderDescriptor` | Stores canonical ID, aliases, and automatic priority |
| `ProviderRegistry<S>` | Owns shared runtime registration and default selection state |
| `ProviderSelection` | Describes candidates and the creation fallback policy |
| `ResolvingServiceProvider<S>` | Owns the resolved candidate snapshot and creates the service |

The generic `S` keeps providers for unrelated service families from being
mixed. A MIME provider cannot be registered in a filesystem Registry because
their `ServiceSpec` types differ.

## Defining a Service Family

Start with the business capability. It should contain operations consumers call
after initialization. Construction settings belong in a separate config type.

```rust
use std::sync::Arc;

use qubit_spi::ServiceSpec;

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

/// Connects the Greeter configuration and output types to Qubit SPI.
struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    // Input accepted by Greeter providers during service creation.
    type Config = GreeterConfig;
    // Service object returned to consumers after successful creation.
    type Output = Arc<dyn Greeter>;
}
```

`ServiceSpec::Output` is the complete value consumers need. Common choices are
`Arc<dyn Trait>`, a concrete client, or a lightweight handle. Qubit SPI does not
wrap successful outputs with provider metadata and does not cache them.

`ServiceSpec::Config` may be unsized. `create_default()` is available only when
that config implements `Default`; `create(&config)` is always available.

## Implementing a Self-Described Provider

A registrable Provider implements two contracts:

1. `ServiceProvider<S>` for creation behavior.
2. `ProviderDefinition<S>` for stable registration metadata.

```rust
use std::sync::Arc;

use qubit_spi::error::{ProviderCreationError, ProviderError};
use qubit_spi::{
    ProviderDefinition, ProviderDescriptor, ProviderId, ServiceProvider,
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
    fn create(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderCreationError> {
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "the greeting prefix must not be empty",
            )
            .into());
        }
        Ok(Arc::new(FriendlyGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}

impl ProviderDefinition<GreeterSpec> for FriendlyGreeterProvider {
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

`ProviderId` must already be canonical lowercase ASCII. It permits
alphanumeric characters and the separators `-`, `_`, `.`, and `+`, with
alphanumeric endpoints.

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

`ProviderRegistry::builder()` is an optional fluent assembly convenience:

```rust,ignore
let mut builder = ProviderRegistry::<GreeterSpec>::builder();
builder.register(FriendlyGreeterProvider)?;
let registry = builder.build();

// Builder output is still runtime mutable.
registry.register(AnotherProvider)?;
```

Use `register_shared` when the Provider is already stored in an
`Arc<dyn ProviderDefinition<S>>`. Otherwise prefer `register(provider)`.

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

### 1. `lib-greater`: Define the Service and Global Registry

`lib-greater` owns the service contract and the one Registry instance shared
by consumers, providers, and the final App.

```rust
// lib-greater/src/lib.rs
use std::sync::{Arc, LazyLock};

use qubit_spi::{ProviderRegistry, ServiceSpec};

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

/// Connects the Greeter configuration and output types to Qubit SPI.
pub struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    // Input accepted by Greeter providers during service creation.
    type Config = GreeterConfig;
    // Service object returned to consumers after successful creation.
    type Output = Arc<dyn Greeter>;
}

/// Process-wide Greeter provider registry shared by the App and all libraries.
pub static GREETER_REGISTRY: LazyLock<ProviderRegistry<GreeterSpec>> =
    LazyLock::new(ProviderRegistry::default);
```

### 2. `lib-foo`: Consume the Default Service

`lib-foo` depends on `lib-greater` and `qubit-spi`, but not on any concrete
Greeter implementation.

```rust
// lib-foo/src/lib.rs
use lib_greater::GREETER_REGISTRY;
use qubit_spi::ServiceProvider;

/// Creates the App-selected default Greeter and prints one greeting.
pub fn foo() -> Result<(), Box<dyn std::error::Error>> {
    let provider = GREETER_REGISTRY.resolve_default()?;
    let greeter = provider.create_default()?;
    println!("{}", greeter.greet("Rust"));
    Ok(())
}
```

### 3. `lib-friend-greater`: Supply a Third-Party Provider

`lib-friend-greater` depends on `lib-greater` and `qubit-spi`. It implements
the Greeter contract and publishes a self-described provider, but it does not
modify global state by registering itself.

```rust
// lib-friend-greater/src/lib.rs
use std::sync::Arc;

use lib_greater::{Greeter, GreeterConfig, GreeterSpec};
use qubit_spi::error::ProviderCreationError;
use qubit_spi::{
    ProviderDefinition, ProviderDescriptor, ProviderId, ServiceProvider,
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
    fn create(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderCreationError> {
        Ok(Arc::new(FriendlyGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}

impl ProviderDefinition<GreeterSpec> for FriendlyGreeterProvider {
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
use lib_friend_greater::FriendlyGreeterProvider;
use lib_greater::GREETER_REGISTRY;
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
same `GREETER_REGISTRY` from `lib-greater`; neither `lib-foo` nor
`lib-greater` depends on `lib-friend-greater`.

Startup ordering matters: configure the global Registry before downstream code
first requests the service. A `ResolvingServiceProvider` already obtained by a
consumer remains a point-in-time snapshot; later registrations affect future
resolutions, not that existing snapshot.

Cargo normally unifies compatible versions of `lib-greater`. If incompatible
versions are linked simultaneously, each crate version owns a separate static
Registry. The App and `lib-foo` must use the same linked `lib-greater` instance
to share the singleton.

## Selecting Providers

Selection is a value object. It can come from a config file, command-line input,
hard-coded library requirements, or an App default. It is not required to live
inside the service's config type.

### Named Selection

```rust,ignore
let selection = ProviderSelection::named("friendly")?;
let provider = registry.resolve(&selection)?;
```

Named selection resolves exactly one canonical ID or alias. An unknown selector
returns `ProviderSelectionError::UnknownProvider`. Because it contains one
candidate, its fallback policy never causes another Provider to run.

### Ordered Chain

```rust,ignore
let selection = ProviderSelection::chain([
    "remote-greeter",
    "friendly",
    "minimal",
])?;
let provider = registry.resolve(&selection)?;
```

Chain order is caller order. Unknown selectors are skipped. If multiple
selectors refer to the same Provider through its ID and aliases, that Provider
appears once at its first position. Resolution fails with `NoCandidates` only
when no chain entry matches.

### Automatic Selection

```rust,ignore
let provider = registry.resolve(&ProviderSelection::auto())?;
```

Automatic selection includes every registered Provider in deterministic order:

1. priority descending;
2. canonical ID ascending for equal priority.

An empty Registry returns `ProviderSelectionError::EmptyRegistry`.

### Registry Default Selection

A new Registry defaults to `ProviderSelection::auto()` with
`FallbackPolicy::OnAbsence`. The App may replace that value at runtime:

```rust,ignore
let default = ProviderSelection::chain(["remote", "friendly"])?
    .with_fallback_policy(FallbackPolicy::OnAbsence);
registry.set_default_selection(default);

let snapshot = registry.default_selection();
let provider = registry.resolve_default()?;
```

`set_default_selection` stores a validated selection but does not require its
providers to exist yet. This permits setting policy before registration.
`resolve_default` evaluates the current selection against the current Registry.

### Selection and Config Are Independent

These are all valid:

```rust,ignore
// Registry default selection and default config.
let service = registry.resolve_default()?.create_default()?;

// Explicit selection and default config.
let service = registry.resolve(&selection)?.create_default()?;

// Registry default selection and explicit config.
let service = registry.resolve_default()?.create(&config)?;

// Explicit selection and explicit config.
let service = registry.resolve(&selection)?.create(&config)?;
```

Do not force a Provider selection field into every service config. A config may
offer a selection as one convenience source, but Registry APIs remain usable by
callers that have no config object.

## Creating the Service

`ProviderRegistry::resolve` returns `ResolvingServiceProvider<S>`. This type is
a composing `ServiceProvider<S>`: it owns candidate handles and applies the
selection's fallback policy when `create` is called.

```rust,ignore
use qubit_spi::ServiceProvider;

let provider = registry.resolve(&selection)?;
let service = provider.create(&config)?;
```

Import the `ServiceProvider` trait so its methods are in scope.

Successful creation returns `S::Output` directly. If consumers need the chosen
Provider ID on success, that is an observation concern for the domain layer,
not part of the generic service value. Failure diagnostics already retain every
actual attempt needed for error handling.

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

Fallback is evaluated after a Provider returns a leaf `ProviderError`. A
registered Provider should normally return `ProviderCreationError::Provider`
by converting its `ProviderError` with `.into()`.

## Error Model

Errors follow the three lifecycle stages plus input validation.

### Definition and Registration Errors

- `ProviderIdError`: a canonical ID is empty or noncanonical.
- `ProviderSelectorError`: normalized user/config input is empty or invalid.
- `ProviderDescriptorError`: an alias is invalid, duplicated, or matches ID.
- `RegistrationError`: an ID or alias conflicts with Registry state.

### Selection Errors

`ProviderSelectionError` is returned before any Provider is invoked:

- `InvalidSelector`: raw selection input is invalid;
- `EmptyChain`: caller supplied no chain entries;
- `UnknownProvider`: named selection matched nothing;
- `NoCandidates`: no entry in a nonempty chain matched;
- `EmptyRegistry`: automatic selection has no Provider.

These errors contain no Provider creation attempts because none occurred.

### Leaf Provider Errors

One concrete Provider reports a `ProviderError` classified by
`ProviderErrorKind`:

- `Unsupported`: provider cannot serve this request;
- `Unavailable`: provider or required environment is absent;
- `InvalidConfiguration`: provider rejects the supplied config;
- `InitializationFailed`: provider failed unexpectedly while constructing.

Use the `_with_source` constructors to retain an underlying error. Registry
internals do not log or collect external observations; consumers receive a
complete error chain when an operation fails.

### Aggregate Creation Errors

`ProviderCreationError` has two shapes:

- `Provider(error)` for a direct Provider invocation;
- `NoProviderSucceeded { attempts, termination }` for composing creation.

Every `ProviderAttemptFailure` contains the canonical ID and original
`ProviderError` of an actually invoked Provider. Missing chain selectors do not
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
    Some(ProviderCreationTermination::Exhausted) => { /* ... */ }
    Some(ProviderCreationTermination::StoppedByPolicy) => { /* ... */ }
    None => { /* direct Provider error */ }
    _ => { /* future non-exhaustive variant */ }
}
```

`decisive_attempt()` returns the single attempt that directly explains a
policy stop or singleton exhaustion. Multi-candidate exhaustion intentionally
has no single decisive source.

## Concurrency and Snapshot Semantics

`ProviderRegistry<S>` is `Send + Sync` when its stored Provider definitions are,
which the Provider traits require. Its shared state uses an `RwLock`.

- Registration obtains a write lock only after calling `descriptor()`.
- Default-selection replacement obtains a short write lock.
- Resolution obtains a read lock while copying candidate handles.
- Provider creation occurs after the lock is released.
- Poisoned locks recover the retained state instead of panicking again.

A resolved provider owns `Arc` handles to its candidates, so it remains usable
after the Registry is cloned, changed, or dropped. It does not see later
registrations. Resolve again to obtain a new snapshot.

## Recommended Practices

1. Define one `ServiceSpec` per independently selectable service family.
2. Let the domain crate own the service trait and optional global facade.
3. Make each registrable Provider implement `ProviderDefinition` directly.
4. Register App-specific providers before downstream service use begins.
5. Store default policy in the Registry; pass explicit selection only when the
   caller has a real requirement.
6. Keep selection independent from service configuration.
7. Prefer `OnAbsence`; justify `OnAnyError` at the call site.
8. Return classified `ProviderError` values with causal sources.
9. Cache expensive service outputs outside Qubit SPI.
10. Use isolated registries in tests that mutate registrations or defaults.

## Troubleshooting

### A registered provider cannot be found

Check the canonical ID and normalized aliases returned by `descriptor()`. Use
`registry.provider_ids()` and `registry.descriptors()` to inspect snapshots.
Remember that `ProviderId` is not normalized, while `ProviderSelector` is.

### `resolve_default()` chooses an unexpected provider

Inspect `registry.default_selection()`. A new Registry defaults to automatic
selection, which uses priority descending and canonical ID ascending. If App
startup should pick one provider, call `set_default_selection` explicitly.

### Fallback does not continue

Inspect the terminal attempt's `ProviderErrorKind` and the selection's policy.
`OnAbsence` stops on `InvalidConfiguration` and `InitializationFailed` by
design. Named selection has no second candidate.

### A newly registered provider is not visible

Existing Registry clones see new registrations, but an already resolved
`ResolvingServiceProvider` is a snapshot. Resolve again. For global facades,
also confirm App and library link the same domain-crate version.

### `create_default()` is unavailable

`S::Config` must implement `Default`, and `ServiceProvider` must be imported.
Otherwise construct a config and call `create(&config)`.

### Duplicate registration fails during repeated tests

Process-wide registries intentionally retain state. Prefer an isolated
`ProviderRegistry::default()` per test, or run the global mutation scenario in
an isolated process.

## API Reference

| API | Purpose |
| --- | --- |
| `ServiceSpec` | Bind config and output types |
| `ServiceProvider::create` | Create with explicit config |
| `ServiceProvider::create_default` | Create with `Config::default()` |
| `ProviderDefinition::descriptor` | Self-describe a registrable Provider |
| `ProviderRegistry::register` | Register an owned Provider at runtime |
| `ProviderRegistry::register_shared` | Register an existing shared Provider |
| `ProviderRegistry::set_default_selection` | Replace the process/component default policy |
| `ProviderRegistry::resolve` | Resolve an explicit selection |
| `ProviderRegistry::resolve_default` | Resolve the current Registry default |
| `ProviderRegistry::descriptors` | Snapshot registration metadata |
| `ProviderRegistry::provider_ids` | Snapshot canonical IDs |
| `ProviderSelection::named` | Select exactly one ID or alias |
| `ProviderSelection::chain` | Select caller-ordered candidates |
| `ProviderSelection::auto` | Select all providers deterministically |
| `ProviderSelection::with_fallback_policy` | Attach creation fallback policy |
| `ResolvingServiceProvider` | Create through a resolved candidate snapshot |

For exact signatures and non-exhaustive error variants, use the
[generated API documentation](https://docs.rs/qubit-spi).
