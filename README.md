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

- Strongly typed provider registries for one service trait, config type, and
  provider error type.
- Stable provider ids and case-insensitive aliases.
- Runtime availability checks for optional backends.
- Priority-based automatic provider selection.
- Explicit default plus fallback-chain selection.
- Shared provider registration through `Arc`.
- Error details that preserve unknown, unavailable, and creation-failure
  candidate states.

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
    ProviderRegistry,
    ProviderRegistryError,
    ServiceProvider,
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
struct EnglishProvider;

impl ServiceProvider for EnglishProvider {
    type Config = ();
    type Service = dyn Greeter;

    fn id(&self) -> &'static str {
        "english"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["en"]
    }

    fn create(&self, _config: &Self::Config) -> Result<Box<Self::Service>, ProviderRegistryError> {
        Ok(Box::new(EnglishGreeter))
    }
}

let mut registry = ProviderRegistry::<dyn Greeter, ()>::new();
registry
    .register(EnglishProvider)
    .expect("provider names should be unique");

let greeter = registry
    .create("en", &())
    .expect("registered provider should create a greeter");
assert_eq!("hello", greeter.greet());
```

## Core Concepts

### ServiceProvider

`ServiceProvider` is the factory contract implemented by each backend. A
provider supplies:

| Method | Purpose |
| --- | --- |
| `id()` | Canonical stable provider id |
| `aliases()` | Additional names accepted by the registry |
| `priority()` | Higher value wins during automatic selection |
| `availability(config)` | Runtime check for optional dependencies |
| `create(config)` | Creates a boxed service implementation |

The associated `Service` type can be a trait object such as `dyn Greeter`.

### ProviderRegistry

`ProviderRegistry<S, C>` stores providers for one service type `S` and one
configuration type `C`.

Provider ids and aliases are matched case-insensitively. Duplicate names are
rejected during registration, including conflicts among a provider's own id and
aliases.

### ProviderSelection

`ProviderSelection` describes how `create_default()` chooses candidates:

- default name is empty or `auto`: try registered providers by descending
  priority, then by provider id.
- default name is explicit: try the default first, then configured fallbacks in
  order.

Selection stops at the first provider that is available and successfully creates
a service.

## Fallback Example

```rust
use std::fmt::Debug;

use qubit_spi::{
    ProviderRegistry,
    ProviderRegistryError,
    ProviderSelection,
    ServiceProvider,
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
struct Provider(&'static str, i32);

impl ServiceProvider for Provider {
    type Config = ();
    type Service = dyn Greeter;

    fn id(&self) -> &'static str {
        self.0
    }

    fn priority(&self) -> i32 {
        self.1
    }

    fn create(&self, _config: &()) -> Result<Box<Self::Service>, ProviderRegistryError> {
        Ok(Box::new(GreeterImpl(self.0)))
    }
}

let mut registry = ProviderRegistry::<dyn Greeter, ()>::new();
registry
    .register(Provider("repository", 0))
    .expect("unique provider");
registry
    .register(Provider("native", 10))
    .expect("unique provider");

let selection = ProviderSelection::from_names("native", &["repository"]);
let greeter = registry
    .create_default(&selection, &())
    .expect("one provider should create a greeter");

assert_eq!("native", greeter.greet());
```

## Error Model

`ProviderRegistryError` separates registration, lookup, and selection failures:

| Variant | Meaning |
| --- | --- |
| `EmptyProviderName` | A provider id, alias, or selector was empty |
| `DuplicateProviderName` | A provider id or alias conflicts with another name |
| `UnknownProvider` | No provider matched the requested selector |
| `ProviderUnavailable` | The selected provider reported unavailable |
| `ProviderCreate` | The selected provider failed during creation |
| `NoAvailableProvider` | Every candidate in a fallback chain failed |
| `EmptyRegistry` | Automatic/default creation was requested from an empty registry |

`NoAvailableProvider` keeps ordered `ProviderFailure` values so callers can
explain the whole fallback chain.

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
| `ServiceProvider` | Provider trait implemented by each backend |
| `ProviderRegistry::new()` | Creates an empty registry |
| `ProviderRegistry::register(provider)` | Registers an owned provider |
| `ProviderRegistry::register_arc(provider)` | Registers a shared provider |
| `ProviderRegistry::find_provider(name)` | Resolves a provider by id or alias |
| `ProviderRegistry::create(name, config)` | Creates one service by provider name |
| `ProviderRegistry::create_auto(config)` | Creates a service by automatic priority |
| `ProviderRegistry::create_default(selection, config)` | Creates from default and fallbacks |
| `ProviderSelection` | Default and fallback candidate configuration |
| `ProviderAvailability` | Provider availability state |
| `ProviderFailure` | One failed candidate in a fallback chain |
| `ProviderRegistryError` | Registry error type |

## Rust Version

This crate uses Rust 2024 edition and requires Rust 1.94 or newer.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
