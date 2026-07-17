# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Typed service-provider registration, selection, and creation infrastructure for
Rust applications and libraries.

## Model

`ServiceSpec` defines one service family's configuration and output types.
`ServiceProvider` creates that output, while `ProviderDefinition` adds the
provider's stable identity, aliases, and automatic-selection priority through
`descriptor()`.

`ProviderRegistry` is a cloneable, synchronized catalog. Applications may
register self-described providers during startup or later at runtime. Every
clone observes subsequent registrations and default-selection updates.

Service acquisition has two independent inputs:

1. `ProviderSelection` chooses candidate providers and carries their
   `FallbackPolicy`.
2. `S::Config` configures the service created by the selected provider.

`resolve()` or `resolve_default()` converts current registry state into a
point-in-time `ResolvingServiceProvider` candidate snapshot. Calling `create()`
or `create_default()` on that provider returns `S::Output` directly.

## Installation

```toml
[dependencies]
qubit-spi = "0.8"
```

## Quick Start

```rust
use std::sync::Arc;

use qubit_spi::error::ProviderCreationError;
use qubit_spi::{
    ProviderDefinition,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
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

struct EnglishProvider {
    descriptor: ProviderDescriptor,
}

impl ServiceProvider<GreeterSpec> for EnglishProvider {
    fn create(
        &self,
        _config: &(),
    ) -> Result<Arc<dyn Greeter>, ProviderCreationError> {
        Ok(Arc::new(EnglishGreeter))
    }
}

impl ProviderDefinition<GreeterSpec> for EnglishProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }
}

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let registry = ProviderRegistry::<GreeterSpec>::default();
registry.register(EnglishProvider {
    descriptor: ProviderDescriptor::new(ProviderId::new("english")?)
        .with_aliases(["en"])?
        .with_priority(100),
})?;

registry.set_default_selection(ProviderSelection::named("en")?);
let greeter = registry.resolve_default()?.create_default()?;

assert_eq!("hello", greeter.greet());
# Ok(())
# }
```

## Selection and Fallback

- `ProviderSelection::auto()` snapshots providers by descending priority and
  then ascending canonical provider ID.
- `ProviderSelection::named(...)` resolves one canonical ID or alias.
- `ProviderSelection::chain(...)` preserves configured order, skips unknown
  selectors, and deduplicates aliases that identify the same provider.
- `FallbackPolicy::Never` stops after the first provider creation failure.
- `FallbackPolicy::OnAbsence` continues only after `Unsupported` or
  `Unavailable` leaf failures and is the default policy.
- `FallbackPolicy::OnAnyError` continues after every leaf failure kind.

Selections are immutable values. Use `with_fallback_policy()` to derive a
selection with different fallback behavior. A resolved provider owns a
candidate snapshot: registrations made afterward affect future resolutions,
not an already resolved provider.

## Error Boundaries

Selection and creation fail at different lifecycle stages:

- `ProviderSelectionError` reports invalid selection construction, unknown
  named providers, chains without matching candidates, and empty automatic
  registries. No provider is invoked when this error is returned.
- `ProviderCreationError` reports failures after candidates were selected. A
  leaf `ProviderError` classifies one provider failure. Aggregate creation
  errors retain ordered `ProviderAttemptFailure` values and distinguish
  `Exhausted` from `StoppedByPolicy` termination.

Attempt diagnostics contain only providers that were actually invoked. Error
objects preserve causal source chains. Successful calls return only the service
value; there is no success wrapper or consumer-facing observation API in this
crate.

## Registration and Global Facades

Registration accepts one self-described provider:

```rust,ignore
registry.register(provider)?;
registry.register_shared(shared_provider)?;
```

The registry snapshots `ProviderDefinition::descriptor()` before taking its
write lock. It validates the canonical ID and every alias before mutation, so a
conflicting registration cannot reserve a partial selector set.

This generic crate intentionally defines no global singleton for a concrete
service family. A domain crate can expose a global facade, such as a MIME
detector registry backed by `ProviderRegistry<MimeDetectorSpec>`. The
application can register custom providers through that facade during startup,
while downstream libraries resolve explicit or default selections from the
same shared registry without depending on concrete implementations.

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
