# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

## Why This Crate Exists

Applications usually depend on a capability, not on one concrete
implementation. A MIME subsystem, for example, may use a model-backed detector
when its model is installed, a system command when that command is available,
or a lightweight detector as a fallback.

Without shared infrastructure, each service family tends to repeat the same
startup code: parse a configured name, find a factory, order alternatives,
decide which failures permit fallback, create the service, and preserve enough
context to explain the result. Those handwritten branches are easy to make
inconsistent and hard to diagnose.

Qubit SPI centralizes that lifecycle in a typed, explicitly assembled model. It
does not use global state, and it does not look up untyped objects from a
container.

## What It Provides

- A compile-time contract that gives every Provider in one service family the
  same construction configuration and output service type.
- An immutable Registry assembled explicitly during application startup.
- Named, automatic, and caller-ordered Provider selection.
- Deterministic priority ordering, classified creation errors, controlled
  fallback, and structured attempt diagnostics.
- The canonical ID of the Provider that actually created the service.

## When to Use It

Use Qubit SPI when one capability has multiple interchangeable implementations
and the application must choose among them by configuration, environment, or
fallback rules. Typical examples include MIME detectors, filesystems,
serializers, model backends, and platform-specific adapters.

It is unnecessary for a service with only one implementation. It is also not a
dynamic-library loader, a dependency-injection framework, or a service cache.

## Core Model

| Role | Responsibility |
| --- | --- |
| Service | The application-facing capability that business code calls repeatedly. |
| `ServiceProvider` | A factory that creates one Service implementation from construction configuration. |
| `ServiceSpec` | Binds the shared `Config` type to the complete `Output` service handle. |
| `ProviderDescriptor` | Stores canonical ID, aliases, and priority separately from factory code. |
| `ProviderRegistry` | Holds the immutable catalog of registered Provider factories. |
| `ProviderResolver` | Selects candidates, calls `create`, and applies fallback policy. |
| `CreatedService` | Returns the usable service together with the winning canonical Provider ID. |

The important boundary is: a Provider **creates** a service; the returned
Service then handles business operations.

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
    FallbackPolicy, ProviderDescriptor, ProviderId, ProviderRegistry, ProviderResolver,
    ServiceProvider, ServiceSpec,
};

trait MimeDetector: Send + Sync {
    fn detect(&self, file_name: &str, content: &[u8]) -> &str;
}

struct MimeConfig {
    default_type: String,
}

struct MimeDetectorSpec;

impl ServiceSpec for MimeDetectorSpec {
    type Config = MimeConfig;
    type Output = Arc<dyn MimeDetector>;
}

struct ExtensionDetector {
    default_type: String,
}

impl MimeDetector for ExtensionDetector {
    fn detect(&self, file_name: &str, _content: &[u8]) -> &str {
        if file_name.ends_with(".png") {
            "image/png"
        } else {
            &self.default_type
        }
    }
}

struct ExtensionProvider;

impl ServiceProvider<MimeDetectorSpec> for ExtensionProvider {
    fn create(
        &self,
        config: &MimeConfig,
    ) -> Result<Arc<dyn MimeDetector>, ProviderError> {
        if config.default_type.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "default_type must not be empty",
            ));
        }
        Ok(Arc::new(ExtensionDetector {
            default_type: config.default_type.clone(),
        }))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<MimeDetectorSpec>::builder();
    builder.register(
        ProviderDescriptor::new(ProviderId::new("extension")?),
        ExtensionProvider,
    )?;

    let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
    let config = MimeConfig {
        default_type: "application/octet-stream".to_owned(),
    };
    let created = resolver.create_named("extension", &config)?;

    assert_eq!("extension", created.provider_id().as_str());
    assert_eq!(
        "image/png",
        created.service().detect("photo.png", b"PNG contents"),
    );
    Ok(())
}
```

## How the Example Works

1. `MimeDetector` is the reusable service. File names and content bytes belong
   to its `detect` business operation.
2. `MimeConfig` contains construction-time configuration, while
   `MimeDetectorSpec` requires every Provider to return
   `Arc<dyn MimeDetector>`.
3. `ExtensionProvider::create` validates that configuration and constructs a
   complete detector. It does not detect a file.
4. `ProviderDescriptor` gives that factory the canonical ID `extension`, and
   the Registry stores it in an immutable catalog.
5. `create_named` chooses the Provider and invokes its factory. The returned
   `CreatedService` exposes both the winning ID and the usable detector.
6. Only after creation does the application call `detect("photo.png", ...)`.

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

Resolver calls do not cache service outputs. If service construction is
expensive, create it once during startup and retain or clone the returned
`Arc`.

## Learn More

- Read the [User Guide](doc/user_guide.md) for the complete mental model, a
  detailed annotated example, aliases, priorities, fallback, diagnostics,
  lifecycle, sharing, and performance.
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
