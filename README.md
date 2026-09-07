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
qubit-spi = "0.11"
```

Requires Rust 1.94 or later. This crate has no optional runtime features.

## Quick Start

Create a binary crate, add the dependency above, and put this in `src/main.rs`.
The application registers a provider and chooses it before a consumer resolves
the service. Run `cargo run`; the program prints `Hello, Rust!`.

<!-- spi-example: quick-start; file: app/src/main.rs -->
```rust
use qubit_spi::{ProviderDescriptor, ProviderMetadata, ProviderRegistry, ProviderSelection};
use qubit_spi::{ServiceProvider, ServiceSpec, SyncServiceSpec, provider_descriptor};
use qubit_spi::error::ProviderFailure;

struct Greeting;
impl ServiceSpec for Greeting { type Config = String; type Error = std::io::Error; }
impl SyncServiceSpec for Greeting { type Output = String; }
struct Friendly;
impl ProviderMetadata for Friendly {
    fn descriptor(&self) -> ProviderDescriptor { provider_descriptor!("friendly") }
}
impl ServiceProvider<Greeting> for Friendly {
    fn create_configured(&self, name: &String) -> Result<String, ProviderFailure<std::io::Error>> {
        Ok(format!("Hello, {name}!"))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ProviderRegistry::<Greeting>::default();
    registry.register(Friendly)?;
    registry.set_default_selection(ProviderSelection::named("friendly")?);
    let greeter = registry.resolve()?;
    println!("{}", greeter.create_configured(&"Rust".to_owned())?);
    Ok(())
}
```

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

- [User Guide](doc/user_guide.md): three libraries plus an application, full configuration, fallback and troubleshooting.
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
