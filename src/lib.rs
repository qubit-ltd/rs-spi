// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Qubit SPI
//!
//! Typed service-provider infrastructure for Qubit Rust crates. A service
//! specification chooses a configuration type and output handle. Applications
//! register self-described providers in a shared runtime registry. Downstream
//! code resolves an explicit or default selection into a composing provider,
//! then creates the service with explicit or default configuration.
//!
//! # Example
//!
//! ```rust
//! use std::sync::Arc;
//!
//! use qubit_spi::{
//!     ProviderDefinition,
//!     ProviderDescriptor,
//!     ProviderId,
//!     ProviderRegistry,
//!     ProviderSelection,
//!     ServiceProvider,
//!     ServiceSpec,
//! };
//! use qubit_spi::error::ProviderCreationError;
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
//! struct EnglishProvider {
//!     descriptor: ProviderDescriptor,
//! }
//!
//! impl ServiceProvider<GreeterSpec> for EnglishProvider {
//!     fn create_configured(
//!         &self,
//!         _config: &(),
//!     ) -> Result<Arc<dyn Greeter>, ProviderCreationError> {
//!         Ok(Arc::new(EnglishGreeter))
//!     }
//! }
//!
//! impl ProviderDefinition<GreeterSpec> for EnglishProvider {
//!     fn descriptor(&self) -> ProviderDescriptor {
//!         self.descriptor.clone()
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = ProviderRegistry::<GreeterSpec>::default();
//! registry.register(EnglishProvider {
//!     descriptor: ProviderDescriptor::new(ProviderId::new("english")?),
//! })?;
//! registry.set_default_selection(ProviderSelection::named("english")?);
//!
//! let greeter = registry.resolve()?.create()?;
//! assert_eq!("hello", greeter.greet());
//! # Ok(())
//! # }
//! ```

pub mod error;
mod fallback_policy;
mod internal;
mod provider_creation_termination;
mod provider_definition;
mod provider_descriptor;
mod provider_id;
mod provider_registry;
mod provider_selection;
mod provider_selector;
mod resolving_service_provider;
mod service_provider;
mod service_spec;

pub use fallback_policy::FallbackPolicy;
pub use provider_creation_termination::ProviderCreationTermination;
pub use provider_definition::ProviderDefinition;
pub use provider_descriptor::ProviderDescriptor;
pub use provider_id::ProviderId;
pub use provider_registry::ProviderRegistry;
pub use provider_selection::ProviderSelection;
pub use provider_selector::ProviderSelector;
pub use resolving_service_provider::ResolvingServiceProvider;
pub use service_provider::ServiceProvider;
pub use service_spec::ServiceSpec;
