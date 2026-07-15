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
FallbackPolicy to create a service. The resolver owns its registry handle and
offers read-only access through `registry()` together with its configured
`fallback_policy()`.

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
let created = resolver.create_named("en", &())?;
assert_eq!("hello", created.service().greet());
# Ok(())
# }
~~~

## Selection and failures

- `ProviderSelection::auto()` uses descending descriptor priority and then
  ascending canonical provider ID.
- `ProviderSelection::named(...)` selects exactly one provider.
- `ProviderSelection::chain(...)` tries configuration-provided candidates in order and
  does not attempt the same provider twice through aliases.
- `ProviderResolver::create_auto`, `create_named`, and `create_chain` accept raw
  runtime input and report parsing failures as `ResolutionError` values.
- FallbackPolicy::OnAbsence continues after unknown, unsupported, or
  unavailable optional providers; it stops at invalid configuration and
  initialization failures.
- FallbackPolicy::OnAnyError is available for explicitly best-effort chains.

`ProviderError` classifies a single factory failure. `ResolutionError` records
all attempted candidates, preserves invalid selector input and its validation
source, and distinguishes empty registries and empty raw chains. Its display
text includes ordered attempt diagnostics. Each `AttemptFailure` explicitly
distinguishes an unknown selector from a provider creation error.

Validation and assembly errors are separated by lifecycle:
`ProviderIdError`, `ProviderSelectorError`, `ProviderDescriptorError`,
`ProviderSelectionError`, and `RegistrationError`. Registration errors now
represent registry conflicts only.

`CreatedService` exposes the winning canonical provider ID and can be consumed
through `into_service()` or `into_parts()`. `ProviderRegistry::len()` and
`is_empty()` expose catalog size without allocation.

## Registration

Provider identity belongs to registration rather than to the provider factory.
Use `register(descriptor, provider)` for an owned provider and
`register_shared(descriptor, provider)` when the factory is already held in an
`Arc`. Registration validates every canonical ID and alias before mutating the
builder, so a rejected registration never reserves a partial set of selectors.

The core exports no global registry. Applications explicitly assemble the
providers they need during startup and share the resulting immutable registry
or resolver.
