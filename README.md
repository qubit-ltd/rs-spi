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

### 1. Service contract and shared registry

`lib-greeter` owns the business interface and one `LazyLock` registry. All
participants use the same service-family type and registry instance.

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
use qubit_spi::{ProviderDescriptor, ProviderId, ProviderMetadata, ServiceProvider};

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
        ProviderDescriptor::new(ProviderId::new("friendly").expect("static provider ID is valid"))
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
    GREETER_REGISTRY.set_default_selection(ProviderSelection::named("friendly")?);
    foo()
}
```

### Run the example

Put the four files in a Cargo workspace and run `cargo run -p app`; the output is
`Hello, Rust!`. The [user guide](doc/user_guide.md#run-the-example) has the
workspace manifest and local dependencies.

## With `inventory`

The same Greeter example can discover linked providers. This adds setup for a
single provider, but keeps application registration code stable as independently
maintained provider crates are added. `lib-foo` remains unchanged. In
`lib-greeter`, declare a collection for `GreeterSpec` and build the shared
registry from it:

```diff
+qubit_spi::declare_sync_provider_inventory! {
+    pub mod providers {
+        spec = crate::GreeterSpec;
+    }
+}

-    LazyLock::new(ProviderRegistry::default);
+    LazyLock::new(|| providers::build_registry().expect("valid Greeter provider inventory"));
```

In `lib-friendly-greeter`, submit a factory for the same provider:

```diff
+qubit_spi::submit_sync_provider! {
+    inventory_entry = lib_greeter::providers::Entry;
+    spec = lib_greeter::GreeterSpec;
+    provider = FriendlyGreeterProvider;
+}
```

The application keeps its provider list in `app/src/greeter_providers.rs`:

```diff
+use lib_friendly_greeter as _;
```

Add one import for each provider crate included in this application. The file
serves as an assembly list, similar in purpose to a Spring XML configuration:
it determines which crates are linked. Each provider still owns its factory
and metadata, and the application still chooses the default. In
`app/src/main.rs`, add `mod greeter_providers;` and remove the
`GREETER_REGISTRY.register(FriendlyGreeterProvider)?` call.

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
