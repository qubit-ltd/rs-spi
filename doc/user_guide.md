# Qubit SPI User Guide

[中文版](user_guide.zh_CN.md) · [README](../README.md) · [Design](design.md)

This guide covers `qubit-spi` 0.11 and Rust 1.94+. It is for domain-library authors,
provider implementers and application authors who compose them. The goal is for
`lib-foo` to obtain a greeting service chosen by its application, without depending
on the concrete provider crate.

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

## Install and Assemble the Example

Create a Cargo workspace with members `lib-greeter`, `lib-foo`,
`lib-friendly-greeter` and `app`. Give each package version `0.1.0`, edition `2024`,
and add `qubit-spi = "0.11"` to each package's dependencies. The workspace root is:

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

The four source files below are complete. From the workspace root run
`cargo run -p app`; successful composition prints `Hello, Rust!`.
The [validation manifest](../tests/fixtures/documentation_examples/scenarios.json)
contains complete member manifests used to compile these exact blocks against
the checkout; replace its SPI path dependency with the published version when
building your own project.

### 1. Service contract and shared registry

`lib-greeter` owns the business interface and one `LazyLock` registry. All participants depend on this same service-family type and registry instance.

<!-- spi-example: three-crates; file: lib-greeter/src/lib.rs -->
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

/// Process-wide Greeter provider registry shared by the App and all libraries.
pub static GREETER_REGISTRY: LazyLock<ProviderRegistry<GreeterSpec>> =
    LazyLock::new(ProviderRegistry::default);
```

### 2. Independent consumer

`lib-foo` resolves the application-defined default and creates the service. It does not choose a concrete backend.

<!-- spi-example: three-crates; file: lib-foo/src/lib.rs -->
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

### 3. Third-party provider

`lib-friendly-greeter` supplies metadata and construction. Merely linking this crate does not register its provider.

<!-- spi-example: three-crates; file: lib-friendly-greeter/src/lib.rs -->
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

### 4. Application composition

The application registers providers and sets the default before invoking library consumers. The singleton is owned by the domain crate, not by SPI.

<!-- spi-example: three-crates; file: app/src/main.rs -->
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
structured errors. Use the same dependency as the quick start and put this block
in `src/main.rs`. Run `cargo run`; success means all assertions pass without stdout.

<!-- spi-example: guide-sync; file: app/src/main.rs -->
```rust
use std::error::Error;
use std::io;
use qubit_spi::{FallbackPolicy, ProviderCreationTermination, ProviderDescriptor, ProviderId};
use qubit_spi::{ProviderMetadata, ProviderRegistry, ProviderSelection, ServiceProvider};
use qubit_spi::{ServiceSpec, SyncServiceSpec};
use qubit_spi::error::{ProviderFailure, ProviderFailureKind};

struct Greeting;
impl ServiceSpec for Greeting { type Config = String; type Error = io::Error; }
impl SyncServiceSpec for Greeting { type Output = String; }
struct Backend { remote: bool }
impl ProviderMetadata for Backend {
    fn descriptor(&self) -> ProviderDescriptor {
        let id = if self.remote { "remote" } else { "local" };
        ProviderDescriptor::new(ProviderId::new(id).expect("valid static ID"))
            .with_aliases([if self.remote { "network" } else { "disk" }])
            .expect("valid static alias")
            .with_priority(if self.remote { 100 } else { 0 })
    }
}
impl ServiceProvider<Greeting> for Backend {
    fn create_configured(&self, name: &String) -> Result<String, ProviderFailure<io::Error>> {
        if self.remote {
            return Err(ProviderFailure::unavailable(io::Error::new(
                io::ErrorKind::NotConnected, "remote greeting service is offline",
            )));
        }
        if name.is_empty() {
            return Err(ProviderFailure::invalid_configuration(io::Error::new(
                io::ErrorKind::InvalidInput, "a name is required",
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

    let error = registry.resolve()?.create_configured(&config).expect_err("remote is absent");
    assert_eq!(ProviderCreationTermination::Exhausted, error.termination());
    assert!(error.is_absence());
    for attempt in error.attempts() {
        assert_eq!("remote", attempt.provider_id().as_str());
        assert_eq!(ProviderFailureKind::Unavailable, attempt.failure().kind());
        assert_eq!(io::ErrorKind::NotConnected, attempt.failure().error().kind());
        assert!(attempt.source().is_some());
        assert!(!attempt.failure().error().to_string().is_empty());
    }
    let stopped = registry.resolve_selected(
        &ProviderSelection::auto().with_fallback_policy(FallbackPolicy::Never),
    )?.create_configured(&config).expect_err("Never stops before local");
    assert_eq!(ProviderCreationTermination::StoppedByPolicy, stopped.termination());
    let failed = snapshot.create_configured(&String::new()).expect_err("local rejects empty name");
    assert_eq!(2, failed.attempts().len());
    assert!(!failed.is_absence());
    assert_eq!("local", failed.decisive_attempt().provider_id().as_str());
    assert!(registry.resolve_selected(&ProviderSelection::chain(["missing", "local"])?).is_err());
    assert_eq!("Hello, Rust!", registry.resolve_selected(
        &ProviderSelection::chain_allowing_missing(["missing", "disk"])?,
    )?.create_configured(&config)?);
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
use qubit_spi::{AsyncProviderRegistry, AsyncServiceProvider, AsyncServiceSpec, ProviderFuture};
use qubit_spi::{ProviderDescriptor, ProviderMetadata, ServiceSpec, provider_descriptor};
use qubit_spi::error::ProviderFailure;

struct Greeting;
impl ServiceSpec for Greeting { type Config = String; type Error = std::io::Error; }
impl AsyncServiceSpec for Greeting { type Output = String; }
struct Friendly;
impl ProviderMetadata for Friendly {
    fn descriptor(&self) -> ProviderDescriptor { provider_descriptor!("friendly") }
}
impl AsyncServiceProvider<Greeting> for Friendly {
    fn create_configured<'a>(&'a self, name: &'a String)
        -> ProviderFuture<'a, Result<String, ProviderFailure<std::io::Error>>> {
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
Registration remains open at runtime. There is no unregister, freeze, service
cache, dynamic-library loading or automatic dependency injection. URI semantics,
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
facade around the registry as shown by the three-crate example. Consult the
[API reference](https://docs.rs/qubit-spi) for exact signatures and error variants,
and the [design](design.md) for lock, snapshot and performance decisions.

From this repository, run `python3 scripts/check-documentation.py` to compile and
run every marked Rust block independently for each language. Then run
`./align-ci.sh` and `SPI_PACKAGE_ALLOW_DIRTY=1 ./ci-check.sh` while reviewing local
uncommitted changes; a clean CI checkout uses `./ci-check.sh` without that opt-in.
[README](../README.md) · [中文指南](user_guide.zh_CN.md)
