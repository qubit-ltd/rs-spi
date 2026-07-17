# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Qubit SPI is typed infrastructure for service-provider registries in Rust.
Applications register self-described providers at startup. Independently
developed libraries resolve an explicit or application-defined default
provider without knowing its concrete implementation, then create the service
with explicit or default configuration.

## Why This Crate Exists

An application often depends on a capability rather than one implementation.
A MIME detector may be backed by a model, a system command, or a repository of
signatures. A filesystem may be local, in memory, or remote. The application
should decide which implementations are installed and preferred; a downstream
library should only ask for the capability it needs.

The complete lifecycle has three different decisions:

1. **Registration:** which provider implementations exist in this process?
2. **Selection:** which registered provider or ordered candidates should be
   considered for this request?
3. **Creation:** can the selected provider create a service with the supplied
   configuration?

Qubit SPI keeps those decisions separate and gives each failure stage its own
error type. This prevents service configuration from becoming an accidental
requirement for provider lookup and prevents selection failures from being
mixed with provider initialization failures.

## What It Provides

- `ServiceSpec` binds one service family's configuration and output types.
- `ServiceProvider` creates the service and returns it directly.
- `ProviderDefinition` adds stable ID, aliases, and priority to a provider.
- `ProviderRegistry` is runtime mutable, thread-safe, and shared by clones.
- `ProviderSelection` contains both its target and its creation fallback policy.
- `ResolvingServiceProvider` is the provider returned by registry resolution;
  it applies fallback while creating the service.
- Separate registration, selection, leaf-provider, and aggregate-creation
  errors retain the context needed when an operation fails.

Qubit SPI does not load dynamic libraries, discover crates automatically, cache
created services, or impose a process-wide singleton. A domain crate can expose
its own global Registry facade when App-to-library sharing is required.

## Core Lifecycle

```text
App startup
  register ProviderDefinition values
  set the Registry's default ProviderSelection
                         │
                         ▼
shared ProviderRegistry<ServiceSpec>
                         │ resolve / resolve_default
                         ▼
ResolvingServiceProvider<ServiceSpec>
                         │ create(config) / create_default()
                         ▼
ServiceSpec::Output
```

| Stage | Main API | Success | Failure |
| --- | --- | --- | --- |
| Registration | `register(provider)` | Provider becomes visible through every Registry clone | `RegistrationError` |
| Selection | `resolve(&selection)` or `resolve_default()` | Candidate snapshot in a `ResolvingServiceProvider` | `ProviderSelectionError` |
| Creation | `create(&config)` or `create_default()` | `ServiceSpec::Output` directly | `ProviderCreationError` |

## Installation

```toml
[dependencies]
qubit-spi = "0.8"
```

Qubit SPI requires Rust 1.94 or later.

## Quick Start

This example uses three independently published libraries and one App. It
separates the service contract, downstream consumer, third-party provider, and
application composition root so the runtime ownership is explicit.

Cargo package names use hyphens below; Rust refers to those crates with
underscores. The `Cargo.toml` files are omitted for brevity.

### 1. `lib-greater`: Define the Service and Global Registry

`lib-greater` owns the service contract. Every consumer and provider uses the
same `GreeterSpec` and the same `GREETER_REGISTRY` singleton from this crate.

```rust
// lib-greater/src/lib.rs
use std::sync::{Arc, LazyLock};

use qubit_spi::{ProviderRegistry, ServiceSpec};

pub trait Greeter: Send + Sync {
    fn greet(&self, name: &str) -> String;
}

#[derive(Clone)]
pub struct GreeterConfig {
    pub prefix: String,
}

impl Default for GreeterConfig {
    fn default() -> Self {
        Self {
            prefix: "Hello".to_owned(),
        }
    }
}

pub struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = GreeterConfig;
    type Output = Arc<dyn Greeter>;
}

pub static GREETER_REGISTRY: LazyLock<ProviderRegistry<GreeterSpec>> =
    LazyLock::new(ProviderRegistry::default);
```

### 2. `lib-foo`: Consume the Default Service

`lib-foo` knows the service contract but not its implementation. `foo()` asks
the shared Registry for its default provider, creates a Greeter with default
configuration, and prints the result.

```rust
// lib-foo/src/lib.rs
use lib_greater::GREETER_REGISTRY;
use qubit_spi::ServiceProvider;

pub fn foo() -> Result<(), Box<dyn std::error::Error>> {
    let provider = GREETER_REGISTRY.resolve_default()?;
    let greeter = provider.create_default()?;
    println!("{}", greeter.greet("Rust"));
    Ok(())
}
```

### 3. `lib-friend-greater`: Supply a Third-Party Provider

`lib-friend-greater` depends on the contract from `lib-greater`, implements the
service, and exports one self-described provider. It does not register itself;
the final App owns that policy decision.

```rust
// lib-friend-greater/src/lib.rs
use std::sync::Arc;

use lib_greater::{Greeter, GreeterConfig, GreeterSpec};
use qubit_spi::error::ProviderCreationError;
use qubit_spi::{
    ProviderDefinition, ProviderDescriptor, ProviderId, ServiceProvider,
};

struct FriendlyGreeter {
    prefix: String,
}

impl Greeter for FriendlyGreeter {
    fn greet(&self, name: &str) -> String {
        format!("{}, {}!", self.prefix, name)
    }
}

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

The App is the composition root. During startup it installs the third-party
provider into the singleton owned by `lib-greater`, makes that provider the
default, and then calls `foo()`.

```rust
// app.rs
use lib_foo::foo;
use lib_friend_greater::FriendlyGreeterProvider;
use lib_greater::GREETER_REGISTRY;
use qubit_spi::ProviderSelection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    GREETER_REGISTRY.register(FriendlyGreeterProvider)?;
    GREETER_REGISTRY
        .set_default_selection(ProviderSelection::named("friendly")?);
    foo()
}
```

The program prints `Hello, Rust!`. `lib-foo` receives the provider selected by
the App even though those two crates do not depend on each other. Their shared
coordination point is the singleton defined by `lib-greater`.

The Registry default and service configuration are independent. A caller with
specific requirements can supply either one without forcing the other:

```rust,ignore
let selection = ProviderSelection::named("friendly")?;
let provider = GREETER_REGISTRY.resolve(&selection)?;
let config = GreeterConfig {
    prefix: "Welcome".to_owned(),
};
let greeter = provider.create(&config)?;
```

## Selection and Fallback

| Selection | Candidate order | Missing selectors |
| --- | --- | --- |
| `ProviderSelection::named("id")` | Exactly one provider | Returns `UnknownProvider` during resolution |
| `ProviderSelection::chain([..])` | Caller order, with duplicate providers removed | Missing entries are skipped; resolution fails if none match |
| `ProviderSelection::auto()` | Priority descending, then canonical ID ascending | Fails when the Registry is empty |

Every selection carries a `FallbackPolicy` used later during creation:

- `Never`: stop after the first provider creation failure.
- `OnAbsence` (default): continue only after `Unsupported` or `Unavailable`.
- `OnAnyError`: continue after every leaf-provider error.

Named selection contains one candidate, so it never falls back. Selection does
not call provider code. Creation operates on a point-in-time candidate snapshot
and does not hold the Registry lock while providers run.

## Error Boundaries

| Error | Boundary | Meaning |
| --- | --- | --- |
| `ProviderIdError` | Provider definition | Canonical ID is invalid |
| `ProviderSelectorError` | Input parsing | Selector cannot be normalized and validated |
| `ProviderDescriptorError` | Provider definition | Alias is invalid or internally duplicated |
| `RegistrationError` | Registration | ID or alias is already owned |
| `ProviderSelectionError` | Selection | No candidate can be resolved |
| `ProviderError` | Leaf creation | One concrete provider reports a classified failure |
| `ProviderCreationError` | Creation | Direct or aggregate creation failure with actual attempts |

Aggregate creation errors contain only providers that were actually invoked.
They also report whether traversal exhausted the candidates or stopped because
the fallback policy rejected continuing. Consumers normally return the error;
they only inspect attempts when failure-specific handling is needed.

## Runtime Registries and Global Facades

`ProviderRegistry` wraps synchronized shared state. Cloning it is cheap, and
registrations or default-selection changes made through one clone are visible
through the others. Descriptor and candidate queries return owned snapshots so
provider code never runs under a Registry lock.

A reusable domain crate can wrap one Registry in a `LazyLock` and expose a
domain-specific `global()` method. This is how an App can install a provider
that a separately published library later receives through `resolve_default()`.
The App must configure that Registry before downstream code first needs the
service. If Cargo links incompatible versions of the domain crate, each linked
crate version owns its own static Registry.

Use `ProviderRegistry::default()` or `ProviderRegistry::builder()` when an
isolated Registry is preferable for tests or scoped components. Builder output
remains runtime mutable.

## Learn More

- Read the [User Guide](doc/user_guide.md) for the full lifecycle, provider
  implementation, runtime sharing, selection semantics, fallback, diagnostics,
  and global-facade pattern.
- Browse the [API reference](https://docs.rs/qubit-spi).
- 阅读[中文说明](README.zh_CN.md)。

## Testing

```bash
# Test the core API
cargo test --no-default-features

# Test every feature and documentation example
cargo test --all-features

# Run the complete project CI checks
./ci-check.sh

# Generate the coverage report
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` followed by
`./ci-check.sh` before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-spi](https://github.com/qubit-ltd/rs-spi)
