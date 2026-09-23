# Qubit SPI User Guide

[README](../README.md) · [简体中文指南](user_guide.zh_CN.md) · [Design](design.md) · [Ecosystem comparison](ecosystem-comparison.md)

This guide covers `qubit-spi` 0.13 and Rust 1.94+. It is for domain-library authors,
provider implementers and application authors who compose them. The goal is for
`lib-foo` to obtain a greeting service chosen by its application, without depending
on the concrete provider crate.

The underlying problem is a dependency and integration mismatch: a reusable
library needs a service, but the application knows which backend fits its
deployment. A direct dependency fixes that choice too early, while hand-written
selection and fallback can be duplicated across consumers. This guide shows how
to keep the service contract in the library, make the choice in the application,
and report provider creation failures through one typed flow.

## The Three Stages

| Stage | Input and result | Failure boundary |
| --- | --- | --- |
| Registration | Provider metadata plus a sync/async factory enter a registry | Duplicate canonical ID or alias; no partial registration |
| Selection | A validated selection becomes an owned candidate resolver | Unknown required selector, empty automatic registry or no candidates |
| Creation | Explicit/default configuration goes to each admitted factory | Typed leaf failures, aggregated with actual provider IDs and termination |

`ServiceSpec` binds `Config` and the domain `Error`; `SyncServiceSpec` and
`AsyncServiceSpec` independently bind their output types. A resolver has inherent
creation methods and returns `ProviderCreationError<E>`. It is not itself a leaf
`ServiceProvider<S>`, whose failures are `ProviderFailure<E>`.

## Explicit Registration: An App-Selected Greeter

The application chooses a `Greeter` provider. `lib-foo` depends on the service
contract and shared registry without depending on the provider crate. The four
files show registration, selection and service creation.

### 1. Service contract

`lib-greeter` owns the business interface. The application creates the registry
and passes it to consumers, so startup errors can be returned normally.

<!-- spi-example: three-crates; file: lib-greeter/src/lib.rs -->
```rust
// lib-greeter/src/lib.rs
use std::{
    error::Error,
    fmt,
    sync::Arc,
};

use qubit_spi::{ServiceSpec, SyncServiceSpec};

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

/// Domain error returned when a Greeter provider cannot create a service.
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
    // Domain error retained by classified provider failures.
    type Error = GreeterError;
}

impl SyncServiceSpec for GreeterSpec {
    // Service object returned to consumers after successful creation.
    type Output = Arc<dyn Greeter>;
}
```

### 2. Independent consumer

`lib-foo` resolves the application-defined default and creates the service. It does not choose a concrete backend.

<!-- spi-example: three-crates; file: lib-foo/src/lib.rs -->
```rust
// lib-foo/src/lib.rs
use lib_greeter::GreeterSpec;
use qubit_spi::{ProviderRegistry, ServiceProvider};

/// Creates the App-selected default Greeter and prints one greeting.
pub fn foo(registry: &ProviderRegistry<GreeterSpec>) -> Result<(), Box<dyn std::error::Error>> {
    let provider = registry.resolve()?;
    let greeter = provider.create()?;
    println!("{}", greeter.greet("Rust"));
    Ok(())
}
```

### 3. Third-party provider

`lib-friendly-greeter` supplies metadata and construction. Merely linking this crate does not register its provider.

<!-- spi-example: three-crates; file: lib-friendly-greeter/src/lib.rs -->
```rust
// lib-friendly-greeter/src/lib.rs
use std::sync::Arc;

use lib_greeter::{Greeter, GreeterConfig, GreeterError, GreeterSpec};
use qubit_spi::error::ProviderFailure;
use qubit_spi::{provider_descriptor, ProviderDescriptor, ProviderMetadata, ServiceProvider};

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
        provider_descriptor!("friendly", priority: 100)
    }
}
```

### 4. Application composition

The application creates the registry, registers providers and sets the default before invoking library consumers.

<!-- spi-example: three-crates; file: app/src/main.rs -->
```rust
// app/src/main.rs
use lib_foo::foo;
use lib_friendly_greeter::FriendlyGreeterProvider;
use lib_greeter::GreeterSpec;
use qubit_spi::{ProviderRegistry, ProviderSelection};

// The application owns the registry and passes it to its library consumer.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ProviderRegistry::<GreeterSpec>::default();
    registry.register(FriendlyGreeterProvider)?;
    registry.set_default_selection(ProviderSelection::named("friendly")?);
    foo(&registry)
}
```

### Run the example

To run these files, create a Cargo workspace with members `lib-greeter`, `lib-foo`,
`lib-friendly-greeter` and `app`. Give each package version `0.1.0`, edition `2024`,
and add `qubit-spi = "0.13"` to each package's dependencies. The workspace root is:

```toml
[workspace]
members = ["lib-greeter", "lib-foo", "lib-friendly-greeter", "app"]
resolver = "3"
```

Add the following local dependencies to the members' `[dependencies]` tables:

| Package | Local dependencies |
| --- | --- |
| lib-greeter | none |
| lib-foo | `lib-greeter = { path = "../lib-greeter" }` |
| lib-friendly-greeter | `lib-greeter = { path = "../lib-greeter" }` |
| app | `lib-greeter`, `lib-foo`, `lib-friendly-greeter`, each with `path = "../<package-name>"` |

The four source files above are complete. From the workspace root run
`cargo run -p app`; successful composition prints `Hello, Rust!`.
The [validation manifest](../tests/fixtures/documentation_examples/scenarios.json)
contains complete member manifests used to compile these exact blocks against
the checkout; replace its SPI path dependency with the published version when
building your own project.

## Link-Time Discovery for the Same Greeter

With `inventory`, the contract declares a collection and the provider submits
a factory. `lib-foo` stays exactly as above. The application links provider
crates and selects a default without calling `register`. These are the complete
alternative source files. This adds setup for one provider, but keeps explicit
registration calls out of the application as more provider crates are added.

### 1. Contract: declare the collection

<!-- spi-example: inventory-greeter; file: lib-greeter/src/lib.rs -->
```rust
// lib-greeter/src/lib.rs
use std::{
    error::Error,
    fmt,
    sync::Arc,
};

use qubit_spi::{ServiceSpec, SyncServiceSpec};

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

/// Domain error returned when a Greeter provider cannot create a service.
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
    // Domain error retained by classified provider failures.
    type Error = GreeterError;
}

impl SyncServiceSpec for GreeterSpec {
    // Service object returned to consumers after successful creation.
    type Output = Arc<dyn Greeter>;
}

/// Collect providers submitted for this Greeter service family.
qubit_spi::declare_sync_provider_inventory! {
    pub mod providers {
        spec = crate::GreeterSpec;
    }
}
```

### 2. Consumer: unchanged

<!-- spi-example: inventory-greeter; file: lib-foo/src/lib.rs -->
```rust
// lib-foo/src/lib.rs
use lib_greeter::GreeterSpec;
use qubit_spi::{ProviderRegistry, ServiceProvider};

/// Creates the App-selected default Greeter and prints one greeting.
pub fn foo(registry: &ProviderRegistry<GreeterSpec>) -> Result<(), Box<dyn std::error::Error>> {
    let provider = registry.resolve()?;
    let greeter = provider.create()?;
    println!("{}", greeter.greet("Rust"));
    Ok(())
}
```

### 3. Provider: submit the factory

<!-- spi-example: inventory-greeter; file: lib-friendly-greeter/src/lib.rs -->
```rust
// lib-friendly-greeter/src/lib.rs
use std::sync::Arc;

use lib_greeter::{Greeter, GreeterConfig, GreeterError, GreeterSpec};
use qubit_spi::error::ProviderFailure;
use qubit_spi::{provider_descriptor, ProviderDescriptor, ProviderMetadata, ServiceProvider};

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
        provider_descriptor!("friendly", priority: 100)
    }
}

// Submit a factory; the application still chooses whether to link this crate.
qubit_spi::submit_sync_provider! {
    inventory_entry = lib_greeter::providers::Entry;
    spec = lib_greeter::GreeterSpec;
    provider = FriendlyGreeterProvider;
}
```

### 4. Application: centralize provider imports

`app/src/greeter_providers.rs` lists the provider crates included in this
application. Add one `use some_greeter_provider as _;` line for each additional
implementation. This small assembly list is similar in purpose to a Spring XML
configuration file: it determines which provider crates reach the binary.
Each provider owns its factory and metadata, while the application still
chooses the default. A Cargo dependency alone does not guarantee that an
otherwise unused provider crate is linked.

<!-- spi-example: inventory-greeter; file: app/src/greeter_providers.rs -->
```rust
use lib_friendly_greeter as _;
```

<!-- spi-example: inventory-greeter; file: app/src/main.rs -->
```rust
// app/src/main.rs
mod greeter_providers;

use lib_foo::foo;
use lib_greeter::providers;
use qubit_spi::ProviderSelection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = providers::build_registry()?;
    registry.set_default_selection(ProviderSelection::named("friendly")?);
    foo(&registry)
}
```

Enable `qubit-spi = { version = "0.13", features = ["inventory"] }` in each
participating crate. The workspace members and local dependencies are the same
as above. Run `cargo run -p app`; it prints `Hello, Rust!`.

`build_registry()` returns inventory conflicts as an error. The application
propagates that startup error with `?` and passes the registry to consumers.

`build_registry()` rejects duplicate IDs or aliases without returning a
partial registry. It returns an unsealed registry, so the application can
register stateful providers before calling `seal()`. Entries are found at link
time; discovery order does not set provider priority or the default. This
mechanism does not load shared libraries at runtime.

## Selection, Configuration and Diagnostics

Registry default selection and service configuration are independent. A new
registry defaults to automatic selection with `OnAbsence`; changing that default
affects later resolutions, not existing candidate snapshots. Use
`resolve_default_snapshot()` when configuration validation depends on the exact
default captured by resolution. Its returned selection and result share one
read snapshot. `resolve()` returns only the result of that operation.

| Selection | Meaning |
| --- | --- |
| `named` | One canonical ID or alias, with no second candidate |
| `chain` | Caller order; any unknown selector rejects the entire selection |
| `chain_allowing_missing` | Ignore unknown selectors; fail if no candidate remains |
| `auto` | All providers, priority descending then canonical ID ascending |

Matched provider identities are deduplicated in chains, even through different
aliases. Unknown selector occurrences remain in diagnostic order, including
repetitions. Selections are opaque validated values; inspect `target()` using
`ProviderSelectionTargetRef` and include a wildcard for non-exhaustive variants.

IDs must already be nonempty lowercase ASCII with alphanumeric endpoints and
only `-`, `_`, `.`, `+` separators. IDs are never trimmed or normalized.
Selectors and descriptor aliases trim surrounding whitespace and lowercase ASCII
before validating that grammar. An alias cannot equal the canonical ID or another
normalized alias. `with_aliases` stops consuming the iterator at its first error.
`provider_descriptor!` validates canonical static literals at compile time.

| Fallback policy | Failures admitting another candidate |
| --- | --- |
| `Never` | None |
| `OnAbsence` | `Unsupported`, `Unavailable` |
| `OnAnyError` | All four leaf failure kinds |

`Unsupported` means the provider cannot serve this request; `Unavailable` means
its dependency/environment is absent. `InvalidConfiguration` rejects accepted
request settings; `InitializationFailed` reports failure after request acceptance.
Providers must classify errors deliberately: reporting every error as absence can
hide broken configuration behind an apparently successful fallback.

The next standalone binary combines aliases, fallback, captured defaults and
structured errors. Add `qubit-spi = "0.13"` and put this block
in `src/main.rs`. Run `cargo run`; success means all assertions pass without stdout.

<!-- spi-example: guide-sync; file: app/src/main.rs -->
```rust
use qubit_spi::error::{ProviderFailure, ProviderFailureKind};
use qubit_spi::{FallbackPolicy, ProviderCreationTermination, ProviderDescriptor, provider_descriptor};
use qubit_spi::{ProviderMetadata, ProviderRegistry, ProviderSelection, ServiceProvider};
use qubit_spi::{ServiceSpec, SyncServiceSpec};
use std::error::Error;
use std::io;

struct Greeting;

impl ServiceSpec for Greeting {
    type Config = String;
    type Error = io::Error;
}

impl SyncServiceSpec for Greeting {
    type Output = String;
}

struct Backend {
    remote: bool,
}

impl ProviderMetadata for Backend {
    fn descriptor(&self) -> ProviderDescriptor {
        if self.remote {
            provider_descriptor!("remote", aliases: ["network"], priority: 100)
        } else {
            provider_descriptor!("local", aliases: ["disk"])
        }
    }
}

impl ServiceProvider<Greeting> for Backend {
    fn create_configured(&self, name: &String) -> Result<String, ProviderFailure<io::Error>> {
        if self.remote {
            return Err(ProviderFailure::unavailable(io::Error::new(
                io::ErrorKind::NotConnected,
                "remote greeting service is offline",
            )));
        }
        if name.is_empty() {
            return Err(ProviderFailure::invalid_configuration(io::Error::new(
                io::ErrorKind::InvalidInput,
                "a name is required",
            )));
        }
        Ok(format!("Hello, {name}!"))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let registry = ProviderRegistry::<Greeting>::default();
    registry.register(Backend { remote: true })?;
    registry.register(Backend { remote: false })?;
    let config = String::from("Rust");
    let chain = ProviderSelection::chain([" NETWORK ", "disk", "local"])?
        .with_fallback_policy(FallbackPolicy::OnAbsence);
    registry.set_default_selection(chain.clone());
    let (captured, result) = registry.resolve_default_snapshot();
    assert_eq!(chain, captured);
    let snapshot = result?;
    registry.set_default_selection(ProviderSelection::named("remote")?);
    assert_eq!("Hello, Rust!", snapshot.create_configured(&config)?);

    let error = match registry.resolve()?.create_configured(&config) {
        Ok(_) => return Err(io::Error::other("expected remote provider to be unavailable").into()),
        Err(error) => error,
    };
    assert_eq!(ProviderCreationTermination::Exhausted, error.termination());
    assert!(error.is_absence());
    for attempt in error.attempts() {
        assert_eq!("remote", attempt.provider_id().as_str());
        assert_eq!(ProviderFailureKind::Unavailable, attempt.failure().kind());
        assert_eq!(
            io::ErrorKind::NotConnected,
            attempt.failure().error().kind()
        );
        assert!(attempt.source().is_some());
        assert!(!attempt.failure().error().to_string().is_empty());
    }
    let stopped = match registry
        .resolve_selected(&ProviderSelection::auto().with_fallback_policy(FallbackPolicy::Never))?
        .create_configured(&config)
    {
        Ok(_) => return Err(io::Error::other("expected Never policy to stop before local").into()),
        Err(error) => error,
    };
    assert_eq!(
        ProviderCreationTermination::StoppedByPolicy,
        stopped.termination()
    );
    let failed = match snapshot.create_configured(&String::new()) {
        Ok(_) => return Err(io::Error::other("expected local provider to reject an empty name").into()),
        Err(error) => error,
    };
    assert_eq!(2, failed.attempts().len());
    assert!(!failed.is_absence());
    assert_eq!("local", failed.decisive_attempt().provider_id().as_str());
    assert!(registry
        .resolve_selected(&ProviderSelection::chain(["missing", "local"])?)
        .is_err());
    assert_eq!(
        "Hello, Rust!",
        registry
            .resolve_selected(&ProviderSelection::chain_allowing_missing([
                "missing", "disk"
            ])?,)?
            .create_configured(&config)?
    );
    Ok(())
}
```

A resolver error contains only actual calls. `attempt.failure()` retains the
classified failure; `attempt.failure().error()` accesses the domain error. Pass
the domain error directly to a `ProviderFailure` constructor. Implement `Error::source`
on that domain type when it has its own cause; SPI preserves the resulting chain.
There are no special `_with_source` constructors.

`Exhausted` means every admitted candidate was attempted. `StoppedByPolicy` means
another candidate existed but policy rejected continuing. A final failure is
`Exhausted` even under `Never`. `decisive_attempt()` is the final actual attempt;
`is_absence()` means all actual failures are absence failures. `into_parts()`
transfers owned diagnostics to consumers. Successful creation does not return the failures that preceded success.
SPI does not log or emit telemetry.

## Asynchronous Creation

Add `futures = "0.3"` to the standalone binary's dependencies for this example.
The library itself does not depend on this executor. Registration and resolution
remain synchronous; only creation is awaited, one candidate at a time.

<!-- spi-example: guide-async; file: app/src/main.rs -->
```rust
use qubit_spi::error::ProviderFailure;
use qubit_spi::{provider_descriptor, ProviderDescriptor, ProviderMetadata, ServiceSpec};
use qubit_spi::{AsyncProviderRegistry, AsyncServiceProvider, AsyncServiceSpec, ProviderFuture};

struct Greeting;

impl ServiceSpec for Greeting {
    type Config = String;
    type Error = std::io::Error;
}

impl AsyncServiceSpec for Greeting {
    type Output = String;
}

struct Friendly;

impl ProviderMetadata for Friendly {
    fn descriptor(&self) -> ProviderDescriptor {
        provider_descriptor!("friendly")
    }
}

impl AsyncServiceProvider<Greeting> for Friendly {
    fn create_configured<'a>(
        &'a self,
        name: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<std::io::Error>>> {
        Box::pin(async move { Ok(format!("Hello, {name}!")) })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = AsyncProviderRegistry::<Greeting>::default();
    registry.register(Friendly)?;
    let resolver = registry.resolve()?;
    let config = String::from("Rust");
    let output = futures::executor::block_on(resolver.create_configured(&config))?;
    assert_eq!("Hello, Rust!", output);
    Ok(())
}
```

`ProviderFuture<'a, T>` is a boxed sendable future borrowing provider/configuration
for `'a`. Async configuration must be `Sync`; async output is `Send + 'static`.
Sync output has no corresponding Send requirement. `Config` may be unsized.
`create()` requires `Config: Default` synchronously, and `Default + Send`
asynchronously; otherwise use `create_configured(&config)`.

Dropping an unpolled resolver future invokes no provider. Dropping it while a
candidate is pending drops that candidate's future and never starts another
candidate. Cancellation is not a `ProviderFailure` and does not trigger fallback.
This does not roll back external effects or stop tasks spawned independently by
a provider. Panics before a leaf returns a future or while polling propagate;
`OnAnyError` does not catch panics.

## Runtime Ownership and Limits

Registry clones share registrations and default changes. Sync and async
registries remain independent. Metadata is captured once per registration attempt,
before taking the write lock. Conflicts are checked after that callback returns;
a failed attempt inserts none of its own selectors. Reentrant changes made by
metadata callbacks remain, even if the callback later panics. Factories, formatting callbacks
and rejected-provider destruction run outside catalog locks.

IDs and descriptors are owned snapshots in successful registration order;
automatic candidates use the separate priority order. Resolvers retain candidate
identity, order and policy, but share provider objects. They do not deep-copy
mutable backend state. Every create calls a factory; that factory may clone an
existing `Arc` or return a handle to a cached client. SPI does not promise a new
underlying resource for every output. Service-operation errors after successful
creation do not reenter fallback.

Use isolated registries in tests and scoped components. A domain `global()` facade
can wrap `LazyLock`; configure it before consumers first resolve services.
A global registry retains registered providers and the resources they own for
the lifetime of the process. Different linked versions of a domain crate have
different static registries.
Registration remains open at runtime until the application calls `seal()`. There is
no unregister or unseal operation, service cache, dynamic-library loading or
automatic dependency injection. URI semantics,
credentials, output identity validation and lifecycle policy belong to domain
crates/providers. Successful output is returned directly without metadata wrapping.

## Troubleshooting

| Symptom | Check and action |
| --- | --- |
| Provider not found | Inspect `provider_ids()`/`descriptors()`; distinguish strict IDs from normalized selectors |
| Unexpected default | Inspect `default_selection()`, priority and canonical tie-break; explicitly set a named selection |
| New registration invisible | Resolve again; old resolvers are snapshots. Check that consumers use the same domain crate and registry |
| Fallback stopped | Inspect failure kind and policy; a named selection has no fallback candidate |
| `create()` not available | Supply explicit configuration; check Default/Send bounds |
| Duplicate registration in tests | Prefer per-test registries or isolate global mutation in a subprocess |
| Unexpected resource sharing | Inspect provider factory caching; SPI invokes it but does not enforce allocation uniqueness |

## Next Steps and Verification

Use `register_shared` for an existing `Arc` provider, or expose a domain-specific
facade around the registry as shown by the explicit registration example. Consult the
[API reference](https://docs.rs/qubit-spi) for exact signatures and error variants,
the [design](design.md) for lock, snapshot and performance decisions, and the
[ecosystem comparison](ecosystem-comparison.md) for scope and trade-offs against related approaches.

From this repository, run `python3 scripts/check-documentation.py` to compile and
run every marked Rust block independently for each language. Then run
`./align-ci.sh` and `SPI_PACKAGE_ALLOW_DIRTY=1 ./ci-check.sh` while reviewing local
uncommitted changes; a clean CI checkout uses `./ci-check.sh` without that opt-in.
[README](../README.md)
