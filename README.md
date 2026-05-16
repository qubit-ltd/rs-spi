# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Documentation](https://docs.rs/qubit-spi/badge.svg)](https://docs.rs/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Typed service provider registry infrastructure for Rust.

## Overview

`qubit-spi` provides a small, explicit SPI layer for crates that define a trait
in one package and allow other packages to provide optional implementations. It
is designed for statically linked Rust crates, where the application decides
which extension crates are linked and when providers are registered.

The public surface is organized around three core types:

- `ServiceSpec`: binds the configuration type and service contract.
- `ServiceProvider`: creates one implementation of a service specification.
- `ProviderRegistry`: stores providers and resolves them by name, fallback
  chain, or priority-based automatic selection.

## Design Goals

- **Explicit Discovery**: applications decide which providers are linked and
  registered.
- **Readable Types**: registries use one `ServiceSpec` type parameter instead
  of separate service, config, and error parameters.
- **Type Safety**: service and configuration types are fixed by the spec at
  compile time.
- **Deterministic Selection**: automatic selection uses stable priority and name
  ordering.
- **Fallback Transparency**: failed candidates are preserved for diagnostics.
- **Small Runtime Surface**: the crate depends only on `log` and `thiserror`.

## Features

- One-parameter registries based on a `ServiceSpec` type.
- Stable dot-free `ProviderName` validation with config-path-safe separators,
  plus normalized provider descriptors.
- Runtime availability checks for optional backends.
- Priority-based automatic provider selection.
- Explicit named provider plus fallback-chain selection.
- Shared provider registration through `Arc`.
- Separated provider creation errors and registry errors.
- Provider creation errors can preserve lower-level source errors.
- Low-noise diagnostics through the `log` facade.

## Installation

Add the crate to `Cargo.toml`:

```toml
[dependencies]
qubit-spi = "0.3"
```

## Quick Start

```rust
use std::fmt::Debug;

use qubit_spi::{
    ProviderCreateError,
    ProviderDescriptor,
    ProviderRegistry,
    ProviderRegistryError,
    ServiceProvider,
    ServiceSpec,
};

trait Greeter: Debug + Send + Sync {
    fn greet(&self) -> &'static str;
}

#[derive(Debug)]
struct EnglishGreeter;

impl Greeter for EnglishGreeter {
    fn greet(&self) -> &'static str {
        "hello"
    }
}

#[derive(Debug)]
struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = ();
    type Service = dyn Greeter;
}

#[derive(Debug)]
struct EnglishProvider;

impl ServiceProvider<GreeterSpec> for EnglishProvider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        ProviderDescriptor::new("english")?.with_aliases(&["en"])
    }

    fn create_box(&self, _config: &()) -> Result<Box<dyn Greeter>, ProviderCreateError> {
        Ok(Box::new(EnglishGreeter))
    }
}

let mut registry = ProviderRegistry::<GreeterSpec>::new();
registry
    .register(EnglishProvider)
    .expect("provider names should be unique");

let greeter = registry
    .create_box("en", &())
    .expect("registered provider should create a greeter");
assert_eq!("hello", greeter.greet());
```

## Core Concepts

### ServiceSpec

`ServiceSpec` binds the configuration type and service contract for one service
family. The contract can be a trait object such as `dyn MyService`; callers then
choose whether a registry returns `Box<dyn MyService>`, `Arc<dyn MyService>`,
or `Rc<dyn MyService>`.

### ServiceProvider

`ServiceProvider<Spec>` is the factory contract implemented by each backend. A
provider supplies:

| Method | Purpose |
| --- | --- |
| `descriptor()` | Stable provider id, aliases, and priority |
| `availability(config)` | Runtime check for optional dependencies |
| `create_box(config)` | Creates a boxed service value |
| `create_arc(config)` | Creates an atomically shared service value |
| `create_rc(config)` | Creates a locally shared service value |

### ProviderRegistry

`ProviderRegistry<Spec>` stores providers for one service specification.

Provider descriptors are captured at registration time. Provider ids and aliases
are normalized into `ProviderName` values and indexed, so lookup is stable even
if a provider instance has mutable internal state.

Provider names are ASCII-only, may contain letters, digits, `_`, and `-`, must
start and end with a letter or digit, and must not contain consecutive
separators. Dots are intentionally rejected so provider names remain safe when
they are used as keys in configuration systems that treat dotted keys as nested
paths.

### ProviderSelection

`ProviderSelection` is an enum:

- `Auto`: try registered providers by descending priority, then provider id.
- `Named`: try a primary provider, then explicit fallbacks in order.

Selection stops at the first provider that is available and successfully creates
a service. Named selections reject duplicated candidate names, and the registry
does not retry a provider that was already attempted through another alias in
the same selection.

## Fallback Example

```rust
use std::fmt::Debug;

use qubit_spi::{
    ProviderCreateError,
    ProviderDescriptor,
    ProviderRegistry,
    ProviderRegistryError,
    ProviderSelection,
    ServiceProvider,
    ServiceSpec,
};

trait Greeter: Debug + Send + Sync {
    fn greet(&self) -> &'static str;
}

#[derive(Debug)]
struct GreeterImpl(&'static str);

impl Greeter for GreeterImpl {
    fn greet(&self) -> &'static str {
        self.0
    }
}

#[derive(Debug)]
struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = ();
    type Service = dyn Greeter;
}

#[derive(Debug)]
struct Provider(&'static str, i32);

impl ServiceProvider<GreeterSpec> for Provider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        Ok(ProviderDescriptor::new(self.0)?.with_priority(self.1))
    }

    fn create_box(&self, _config: &()) -> Result<Box<dyn Greeter>, ProviderCreateError> {
        Ok(Box::new(GreeterImpl(self.0)))
    }
}

let mut registry = ProviderRegistry::<GreeterSpec>::new();
registry
    .register(Provider("repository", 0))
    .expect("unique provider");
registry
    .register(Provider("native", 10))
    .expect("unique provider");

let selection = ProviderSelection::from_names("native", &["repository"])
    .expect("selection names should be valid");
let greeter = registry
    .create_selected_box(&selection, &())
    .expect("one provider should create a greeter");

assert_eq!("native", greeter.greet());
```

## Error Model

Provider errors and registry errors are separate:

- `ProviderCreateError` is returned by a provider factory.
- `ProviderRegistryError` is returned by registration, lookup, and selection.
- `ProviderFailure` records each failed fallback candidate.

`ProviderRegistryError` variants:

| Variant | Meaning |
| --- | --- |
| `EmptyProviderName` | A provider id, alias, or selector was empty |
| `InvalidProviderName` | A provider id, alias, or selector used unsupported characters |
| `DuplicateProviderName` | A provider id or alias conflicts with another name |
| `DuplicateProviderCandidate` | A named selection repeats a normalized candidate name |
| `UnknownProvider` | No provider matched the requested selector |
| `ProviderUnavailable` | The selected provider reported unavailable |
| `ProviderCreate` | The selected provider failed during creation |
| `NoAvailableProvider` | Every candidate in a fallback chain failed |
| `EmptyRegistry` | Automatic/selected creation was requested from an empty registry |

`NoAvailableProvider` keeps ordered `ProviderFailure` values so callers can
explain the whole fallback chain.

`ProviderCreateError::failed_with_source()` and
`ProviderCreateError::unavailable_with_source()` preserve lower-level error
causes through direct registry creation and fallback failure reporting.

## Diagnostics

`qubit-spi` emits low-noise diagnostics through the `log` facade. Applications
remain responsible for installing the logger implementation they prefer. The
crate uses `debug` for successful registration and selection outcomes, and
`trace` for name resolution, candidate ordering, and fallback failures. It does
not log service configuration values or service instances.

## Lifetime Model

`ProviderRegistry` stores providers behind shared trait objects. Registered
providers and service specifications are therefore required to be `'static`.
This is intentional: the crate targets long-lived provider registries assembled
from application crates and extension crates, not short-lived registries that
borrow stack-local provider state.

## Relationship to Java ServiceLoader

Rust does not have a standard-library equivalent of Java `ServiceLoader`.
`qubit-spi` intentionally keeps discovery explicit: extension crates expose a
provider type or registration function, and applications register the providers
they want to make visible. This avoids linker magic and keeps tests isolated.

If a future crate needs linker-time discovery, it can build that layer on top of
`ProviderRegistry` with crates such as `inventory` or `linkme`.

## API Overview

| API | Purpose |
| --- | --- |
| `ServiceSpec` | Binds provider configuration and service contract |
| `ServiceProvider` | Provider trait implemented by each backend |
| `ProviderDescriptor` | Captured provider id, aliases, and priority |
| `ProviderName` | Validated and normalized provider name |
| `ProviderRegistry::new()` | Creates an empty registry |
| `ProviderRegistry::register(provider)` | Registers an owned provider |
| `ProviderRegistry::register_shared(provider)` | Registers a shared `Arc<dyn ServiceProvider<_>>` provider |
| `ProviderRegistry::resolve_provider(name)` | Resolves a provider or returns a precise error |
| `ProviderRegistry::find_provider(name)` | Option-returning provider lookup convenience |
| `ProviderRegistry::iter_provider_names()` | Iterates provider ids without allocation |
| `ProviderRegistry::iter_provider_descriptors()` | Iterates descriptors without allocation |
| `ProviderRegistry::create_box(name, config)` | Creates a boxed service by provider name |
| `ProviderRegistry::create_arc(name, config)` | Creates an atomically shared service by provider name |
| `ProviderRegistry::create_rc(name, config)` | Creates a locally shared service by provider name |
| `ProviderRegistry::create_auto_box(config)` | Creates a boxed service by automatic priority |
| `ProviderRegistry::create_auto_arc(config)` | Creates an atomically shared service by automatic priority |
| `ProviderRegistry::create_auto_rc(config)` | Creates a locally shared service by automatic priority |
| `ProviderRegistry::create_selected_box(selection, config)` | Creates a boxed service from selection |
| `ProviderRegistry::create_selected_arc(selection, config)` | Creates an atomically shared service from selection |
| `ProviderRegistry::create_selected_rc(selection, config)` | Creates a locally shared service from selection |
| `ProviderSelection` | Automatic or named fallback candidate selection |
| `ProviderAvailability` | Provider availability state |
| `ProviderCreateError` | Provider-level creation error |
| `ProviderFailure` | One failed candidate in a fallback chain |
| `ProviderRegistryError` | Registry-level error type |

## Rust Version

This crate uses Rust 2024 edition and requires Rust 1.94 or newer.

## Testing & Code Coverage

This project keeps tests under `tests/` and validates provider name handling,
descriptor normalization, registration, lookup, provider selection, fallback
failure reporting, and error formatting.

### Running Tests

```bash
# Run all tests
cargo test

# Generate a coverage report
./coverage.sh

# Generate a text format coverage report
./coverage.sh text

# Align formatting with CI
./align-ci.sh

# Run CI checks (format, clippy, tests, docs, coverage, audit)
./ci-check.sh
```

## Dependencies

Runtime dependencies are intentionally small:

- `log` provides low-noise diagnostics through the standard logging facade.
- `thiserror` provides concrete error implementations.

## License

Copyright (c) 2026. Haixing Hu.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

<http://www.apache.org/licenses/LICENSE-2.0>

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

See [LICENSE](LICENSE) for the full license text.

## Contributing

Contributions are welcome. Please keep changes aligned with the existing Rust
project structure and run `./ci-check.sh` before opening a pull request.

## Author

**Haixing Hu**

## Related Projects

More Rust libraries from Qubit are published under the
[qubit-ltd](https://github.com/qubit-ltd) GitHub organization.

---

Repository: [https://github.com/qubit-ltd/rs-spi](https://github.com/qubit-ltd/rs-spi)
