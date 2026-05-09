# Qubit SPI

[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg)](https://crates.io/crates/qubit-spi)
[![Documentation](https://docs.rs/qubit-spi/badge.svg)](https://docs.rs/qubit-spi)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Typed service provider registry infrastructure for Rust.

`qubit-spi` provides a small, explicit SPI layer for crates that define a trait
in one package and allow other packages to provide optional implementations. It
is designed for statically linked Rust crates, where the application decides
which extension crates are linked and when providers are registered.

## Features

- One-parameter registries based on a `ServiceSpec` type.
- Stable `ProviderName` validation and normalized provider descriptors.
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
qubit-spi = "0.1"
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
    type Service = Box<dyn Greeter>;
}

#[derive(Debug)]
struct EnglishProvider;

impl ServiceProvider<GreeterSpec> for EnglishProvider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        ProviderDescriptor::new("english")?.with_aliases(&["en"])
    }

    fn create(&self, _config: &()) -> Result<Box<dyn Greeter>, ProviderCreateError> {
        Ok(Box::new(EnglishGreeter))
    }
}

let mut registry = ProviderRegistry::<GreeterSpec>::new();
registry
    .register(EnglishProvider)
    .expect("provider names should be unique");

let greeter = registry
    .create("en", &())
    .expect("registered provider should create a greeter");
assert_eq!("hello", greeter.greet());
```

## Core Concepts

### ServiceSpec

`ServiceSpec` binds the configuration type and provider output type for one
service family. This keeps `ProviderRegistry<Spec>` readable while still
allowing each crate to choose whether providers return `Box<dyn Trait>`,
`Arc<dyn Trait>`, concrete values, or other owned service handles.

### ServiceProvider

`ServiceProvider<Spec>` is the factory contract implemented by each backend. A
provider supplies:

| Method | Purpose |
| --- | --- |
| `descriptor()` | Stable provider id, aliases, and priority |
| `availability(config)` | Runtime check for optional dependencies |
| `create(config)` | Creates the `Spec::Service` service value |

### ProviderRegistry

`ProviderRegistry<Spec>` stores providers for one service specification.

Provider descriptors are captured at registration time. Provider ids and aliases
are normalized into `ProviderName` values and indexed, so lookup is stable even
if a provider instance has mutable internal state.

### ProviderSelection

`ProviderSelection` is an enum:

- `Auto`: try registered providers by descending priority, then provider id.
- `Named`: try a primary provider, then explicit fallbacks in order.

Selection stops at the first provider that is available and successfully creates
a service.

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
    type Service = Box<dyn Greeter>;
}

#[derive(Debug)]
struct Provider(&'static str, i32);

impl ServiceProvider<GreeterSpec> for Provider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        Ok(ProviderDescriptor::new(self.0)?.with_priority(self.1))
    }

    fn create(&self, _config: &()) -> Result<Box<dyn Greeter>, ProviderCreateError> {
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
    .create_selected(&selection, &())
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
| `ServiceSpec` | Binds provider configuration and service types |
| `ServiceProvider` | Provider trait implemented by each backend |
| `ProviderDescriptor` | Captured provider id, aliases, and priority |
| `ProviderName` | Validated and normalized provider name |
| `ProviderRegistry::new()` | Creates an empty registry |
| `ProviderRegistry::register(provider)` | Registers an owned provider |
| `ProviderRegistry::register_shared(provider)` | Registers a shared provider |
| `ProviderRegistry::resolve_provider(name)` | Resolves a provider or returns a precise error |
| `ProviderRegistry::find_provider(name)` | Option-returning provider lookup convenience |
| `ProviderRegistry::iter_provider_names()` | Iterates provider ids without allocation |
| `ProviderRegistry::iter_provider_descriptors()` | Iterates descriptors without allocation |
| `ProviderRegistry::create(name, config)` | Creates one service by provider name |
| `ProviderRegistry::create_auto(config)` | Creates a service by automatic priority |
| `ProviderRegistry::create_selected(selection, config)` | Creates from selection |
| `ProviderSelection` | Automatic or named fallback candidate selection |
| `ProviderAvailability` | Provider availability state |
| `ProviderCreateError` | Provider-level creation error |
| `ProviderFailure` | One failed candidate in a fallback chain |
| `ProviderRegistryError` | Registry-level error type |

## Rust Version

This crate uses Rust 2024 edition and requires Rust 1.94 or newer.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
