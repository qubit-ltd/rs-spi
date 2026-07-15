# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Documentation](https://docs.rs/qubit-spi/badge.svg)](https://docs.rs/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Typed, explicitly assembled service-provider infrastructure for Rust.

## Model

Applications register providers during startup through ProviderRegistryBuilder.
Build produces an immutable, cheaply cloneable ProviderRegistry. A
ProviderResolver combines that catalog with a ProviderSelection and
FallbackPolicy to create a service.

ServiceSpec owns both the configuration and complete output handle. The SPI
core does not convert between Box, Arc, and Rc.

## Installation

~~~toml
[dependencies]
qubit-spi = "0.4"
~~~

## Quick Start

~~~rust
use std::sync::Arc;

use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderError,
    ProviderId,
    ProviderRegistry,
    ProviderResolver,
    ProviderSelection,
    ServiceProvider,
    ServiceSpec,
};

trait Greeter: Send + Sync {
    fn greet(&self) -> &'static str;
}

struct EnglishGreeter;

impl Greeter for EnglishGreeter {
    fn greet(&self) -> &'static str {
        "hello"
    }
}

struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = ();
    type Output = Arc<dyn Greeter>;
}

struct EnglishProvider;

impl ServiceProvider<GreeterSpec> for EnglishProvider {
    fn create(&self, _config: &()) -> Result<Arc<dyn Greeter>, ProviderError> {
        Ok(Arc::new(EnglishGreeter))
    }
}

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut builder = ProviderRegistry::<GreeterSpec>::builder();
builder.register(
    ProviderDescriptor::new(ProviderId::new("english")?).with_aliases(["en"])?,
    EnglishProvider,
)?;
let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
let created = resolver.create(&ProviderSelection::named("en")?, &())?;
assert_eq!("hello", created.service().greet());
# Ok(())
# }
~~~

## Selection and failures

- ProviderSelection::Auto uses descending descriptor priority and then
  ascending canonical provider ID.
- ProviderSelection::Named selects exactly one provider.
- ProviderSelection::Chain tries configuration-provided candidates in order and
  does not attempt the same provider twice through aliases.
- FallbackPolicy::OnAbsence continues after unknown, unsupported, or
  unavailable optional providers; it stops at invalid configuration and
  initialization failures.
- FallbackPolicy::OnAnyError is available for explicitly best-effort chains.

ProviderError classifies a single factory failure. ResolutionError records all
attempted candidates. CreatedService exposes the canonical ID of the provider
that won selection.

## 0.4 migration

| 0.3 API | 0.4 replacement |
| --- | --- |
| ServiceSpec::Service | ServiceSpec::Output |
| create_box, create_arc, create_rc | One ServiceProvider::create; the spec chooses its output handle |
| Provider descriptor() | ProviderDescriptor passed to builder registration |
| availability() | Classified ProviderError returned by create() |
| Mutable ProviderRegistry::register | ProviderRegistryBuilder::register, then build() |
| create_auto_* and create_selected_* | ProviderResolver::create |
| register_default in a domain crate | Explicit application startup assembly |

This release intentionally provides no compatibility layer. Downstream crates
such as qubit-fs, qubit-mime, and qubit-magika migrate in separate changes.
rs-llmsdk-core remains provider-neutral and does not depend on this crate.
