// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Qubit SPI
//!
//! Typed, explicitly assembled service-provider infrastructure for Qubit Rust
//! crates. A service specification chooses a configuration type and complete
//! output handle. Applications build an immutable provider registry during
//! startup, then use a resolver to create services through an explicit
//! selection and fallback policy.
//!
//! # Example
//!
//! ```rust
//! use std::sync::Arc;
//!
//! use qubit_spi::{
//!     FallbackPolicy,
//!     ProviderDescriptor,
//!     ProviderId,
//!     ProviderRegistry,
//!     ProviderResolver,
//!     ServiceProvider,
//!     ServiceSpec,
//! };
//!
//! trait Greeter: Send + Sync {
//!     fn greet(&self) -> &'static str;
//! }
//!
//! struct EnglishGreeter;
//!
//! impl Greeter for EnglishGreeter {
//!     fn greet(&self) -> &'static str {
//!         "hello"
//!     }
//! }
//!
//! struct GreeterSpec;
//!
//! impl ServiceSpec for GreeterSpec {
//!     type Config = ();
//!     type Output = Arc<dyn Greeter>;
//! }
//!
//! struct EnglishProvider;
//!
//! impl ServiceProvider<GreeterSpec> for EnglishProvider {
//!     fn create(
//!         &self,
//!         _config: &(),
//!     ) -> Result<Arc<dyn Greeter>, qubit_spi::error::ProviderError> {
//!         Ok(Arc::new(EnglishGreeter))
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut builder = ProviderRegistry::<GreeterSpec>::builder();
//! builder.register(
//!     ProviderDescriptor::new(ProviderId::new("english")?).with_aliases(["en"])?,
//!     EnglishProvider,
//! )?;
//! let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
//! let created = resolver.create_named("en", &())?;
//! assert_eq!("hello", created.service().greet());
//! # Ok(())
//! # }
//! ```

mod created_service;
pub mod error;
mod fallback_policy;
mod internal;
mod provider_descriptor;
mod provider_id;
mod provider_registry;
mod provider_registry_builder;
mod provider_resolver;
mod provider_selection;
mod provider_selection_kind;
mod provider_selector;
mod resolution_termination;
mod resolved_provider;
mod service_provider;
mod service_spec;

pub use created_service::CreatedService;
pub use fallback_policy::FallbackPolicy;
pub use provider_descriptor::ProviderDescriptor;
pub use provider_id::ProviderId;
pub use provider_registry::ProviderRegistry;
pub use provider_registry_builder::ProviderRegistryBuilder;
pub use provider_resolver::ProviderResolver;
pub use provider_selection::ProviderSelection;
pub use provider_selection_kind::ProviderSelectionKind;
pub use provider_selector::ProviderSelector;
pub use resolution_termination::ResolutionTermination;
pub use resolved_provider::ResolvedProvider;
pub use service_provider::ServiceProvider;
pub use service_spec::ServiceSpec;
