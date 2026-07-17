# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Typed, explicitly assembled service-provider infrastructure for Rust.

## Overview

Qubit SPI lets an application define a service family, register provider
factories during startup, and resolve one provider through automatic, named, or
ordered selection. A built `ProviderRegistry` is immutable and cheaply
cloneable, while `ProviderResolver` applies a configured `FallbackPolicy` when
a provider cannot create the requested service.

The crate owns provider identity and selection metadata, but it does not impose
a service handle type or convert between `Box`, `Arc`, and `Rc`.

## Documentation

- [User Guide](doc/user_guide.md)
- [API Reference](https://docs.rs/qubit-spi)
- [Chinese README](README.zh_CN.md)

## Installation

```toml
[dependencies]
qubit-spi = "0.8"
```

Qubit SPI requires Rust 1.94 or later.

## Quick Start

```rust
use std::sync::Arc;

use qubit_spi::error::ProviderError;
use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderResolver,
    ServiceProvider,
    ServiceSpec,
};

trait Greeter: Send + Sync {
    fn greet(&self) -> &'static str;
}

struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = ();
    type Output = Arc<dyn Greeter>;
}

struct EnglishGreeter;

impl Greeter for EnglishGreeter {
    fn greet(&self) -> &'static str {
        "hello"
    }
}

struct EnglishProvider;

impl ServiceProvider<GreeterSpec> for EnglishProvider {
    fn create(&self, _config: &()) -> Result<Arc<dyn Greeter>, ProviderError> {
        Ok(Arc::new(EnglishGreeter))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<GreeterSpec>::builder();
    builder.register(
        ProviderDescriptor::new(ProviderId::new("english")?)
            .with_aliases(["en"])?
            .with_priority(100),
        EnglishProvider,
    )?;

    let resolver = ProviderResolver::new(
        builder.build(),
        FallbackPolicy::OnAbsence,
    );
    let created = resolver.create_named("en", &())?;

    assert_eq!("english", created.provider_id().as_str());
    assert_eq!("hello", created.service().greet());
    Ok(())
}
```

## Common Selection Modes

- `create_auto` tries providers by descending priority and then ascending
  canonical provider ID.
- `create_named` resolves one canonical ID or alias and never falls back.
- `create_chain` tries selectors in caller-provided order, records unknown
  selectors, and does not invoke the same provider twice through aliases.
- `FallbackPolicy::OnAbsence` continues after unsupported or unavailable
  providers. `FallbackPolicy::OnAnyError` continues after every provider
  creation error.

See the [User Guide](doc/user_guide.md) for reusable validated selections,
registry lookup, complete fallback semantics, error diagnostics, concurrency,
and recommended practices.

## Testing

```bash
# Core API with the default empty feature set
cargo test --no-default-features

# Core API plus regex validation
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
