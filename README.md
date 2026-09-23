# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Qubit SPI lets a reusable Rust library obtain an application-selected service
without depending on a concrete backend. Applications register available
providers, choose a default, and pass configuration when creating the service.
For example, a library can request a greeting service while its application
installs the implementation appropriate to its deployment.

## Installation

```toml
[dependencies]
qubit-spi = "0.13"
```

Requires Rust 1.94 or later. The default feature set is empty.

## Example: An App-Selected Greeter

`lib-foo` needs a Greeter, but the application chooses the implementation.
The four files show the service contract, consumer, provider and application.

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

Put the four files in a Cargo workspace and run `cargo run -p app`; the output is
`Hello, Rust!`. The [user guide](doc/user_guide.md#run-the-example) has the
workspace manifest and local dependencies.

## With `inventory`

The same Greeter example can discover linked providers. `lib-foo` stays
unchanged. Each provider crate submits a factory, and the application builds
one registry from the providers it links.

In `lib-greeter/src/lib.rs`, declare the collection:

```rust
qubit_spi::declare_sync_provider_inventory! {
    pub mod providers {
        spec = crate::GreeterSpec;
    }
}
```rust

In `lib-friendly-greeter/src/lib.rs`, submit its factory:

```rust
qubit_spi::submit_sync_provider! {
    inventory_entry = lib_greeter::providers::Entry;
    spec = lib_greeter::GreeterSpec;
    provider = FriendlyGreeterProvider;
}
```rust

List provider crates in `app/src/greeter_providers.rs`:

```rust
use lib_friendly_greeter as _;
```rust

Add one import for each provider crate included in the application. This file
serves as an assembly list, similar in purpose to a Spring XML configuration:
it determines which crates are linked. Each provider still owns its factory
and metadata, and the application still chooses the default. A Cargo dependency
alone does not guarantee that an otherwise unused provider crate is linked.

In `app/src/main.rs`, include that file and build the registry. This replaces
the explicit `register` call from the first example:

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
```rust

Enable `qubit-spi = { version = "0.13", features = ["inventory"] }` in the
participating crates. The [complete runnable version](doc/user_guide.md#link-time-discovery-for-the-same-greeter)
shows all files and explains how to handle inventory conflicts.

## When to Use It

Use a registry when reusable libraries should share an application-selected
implementation, or when a deployment can choose among several backends.
A single concrete implementation with no selection needs can use an ordinary
constructor. SPI manages provider registration and construction; domain APIs,
resource caching and provider-specific configuration validation stay in your
service crate.

## Core Capabilities

- Typed service families keep unrelated providers separate.
- Canonical IDs and normalized aliases support named, ordered-chain and automatic selection.
- Explicit fallback policies distinguish absent backends from configuration or initialization failures.
- Sync and async registries expose the same catalog operations; async creation uses sendable futures without selecting an executor.
- Registry clones share runtime changes; resolved candidates retain their captured identity, order and policy.
- Typed errors retain actual attempts and domain diagnostics. Each create invokes a factory, which may reuse an existing resource.

## Learn More

- [User Guide](doc/user_guide.md): complete explicit and inventory Greeter examples, configuration, fallback and troubleshooting.
- [中文用户指南](doc/user_guide.zh_CN.md).
- [Design](doc/design.md) and [中文设计说明](doc/design.zh_CN.md): contracts and implementation decisions.
- [API reference](https://docs.rs/qubit-spi).
- [中文 README](README.zh_CN.md).

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
./ci-check.sh

# Check code coverage
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` to format code and
`./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-spi](https://github.com/qubit-ltd/rs-spi)
