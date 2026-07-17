# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

## What This Crate Does

An application can register several implementations of one service and select
the appropriate implementation at runtime without global state or untyped
lookup.

For example, an application can prefer a cloud backend, fall back to a local
backend when the cloud is unavailable, or select one backend by configuration.
Rust checks that every provider accepts the same configuration type and returns
the same output type.

## Installation

```toml
[dependencies]
qubit-spi = "0.8"
```

Qubit SPI requires Rust 1.94 or later.

## Quick Start

```rust
use qubit_spi::error::ProviderError;
use qubit_spi::{
    FallbackPolicy, ProviderDescriptor, ProviderId, ProviderRegistry, ProviderResolver,
    ServiceProvider, ServiceSpec,
};

struct GreetingSpec;

impl ServiceSpec for GreetingSpec {
    type Config = ();
    type Output = &'static str;
}

struct EnglishProvider;

impl ServiceProvider<GreetingSpec> for EnglishProvider {
    fn create(&self, _config: &()) -> Result<&'static str, ProviderError> {
        Ok("hello")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder.register(
        ProviderDescriptor::new(ProviderId::new("english")?),
        EnglishProvider,
    )?;

    let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
    let created = resolver.create_named("english", &())?;

    assert_eq!("english", created.provider_id().as_str());
    assert_eq!("hello", *created.service());
    Ok(())
}
```

## How the Example Works

1. `GreetingSpec` fixes the provider input as `()` and the output as
   `&'static str`.
2. `EnglishProvider` implements the factory operation that returns the
   greeting.
3. `ProviderDescriptor` assigns the canonical name `english` during
   registration.
4. `ProviderRegistry::builder()` collects providers during startup, and
   `build()` freezes the catalog for runtime use.
5. `ProviderResolver::create_named` selects `english`; the returned
   `CreatedService` contains both the output and the winning canonical ID.

## Common Selection Modes

| Need | Method | Behavior |
| --- | --- | --- |
| One configured provider | `create_named` | Tries exactly one canonical ID or alias; never falls back. |
| Best available provider | `create_auto` | Uses priority descending, then canonical ID ascending. |
| Ordered preferences | `create_chain` | Tries selectors in caller order and avoids invoking one provider twice through aliases. |

Every resolver has a fallback policy. `FallbackPolicy::OnAbsence` is the safer
default: it continues after unsupported or unavailable providers but stops on
configuration and initialization errors. Use `OnAnyError` only when
best-effort fallback is intentional.

## Learn More

- Read the [User Guide](doc/user_guide.md) for a complete annotated example and
  details about realistic output handles, aliases, priorities, fallback,
  diagnostics, sharing, and performance.
- Browse the [API reference](https://docs.rs/qubit-spi).
- 阅读[中文说明](README.zh_CN.md)。

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
