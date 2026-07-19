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
//!     ProviderDescriptor,
//!     ProviderId,
//!     ProviderMetadata,
//!     ProviderRegistry,
//!     ProviderSelection,
//!     ServiceProvider,
//!     ServiceSpec,
//!     SyncServiceSpec,
//! };
//! use qubit_spi::error::ProviderError;
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
//! }
//!
//! impl SyncServiceSpec for GreeterSpec {
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
//!     ) -> Result<Arc<dyn Greeter>, ProviderError> {
//!         Ok(Arc::new(EnglishGreeter))
//!     }
//! }
//!
//! impl ProviderMetadata for EnglishProvider {
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
//!
//! The asynchronous Registry has the same synchronous registration and
//! resolution stages. Only creation is awaited, and providers return the
//! runtime-independent [`ProviderFuture`]:
//!
//! ```rust
//! use std::sync::Arc;
//!
//! use qubit_spi::error::ProviderError;
//! use qubit_spi::{
//!     AsyncProviderRegistry,
//!     AsyncServiceProvider,
//!     AsyncServiceSpec,
//!     ProviderDescriptor,
//!     ProviderFuture,
//!     ProviderId,
//!     ProviderMetadata,
//!     ProviderSelection,
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
//! }
//!
//! impl AsyncServiceSpec for GreeterSpec {
//!     type Output = Arc<dyn Greeter>;
//! }
//!
//! struct EnglishProvider;
//!
//! impl AsyncServiceProvider<GreeterSpec> for EnglishProvider {
//!     fn create_configured<'a>(
//!         &'a self,
//!         _config: &'a (),
//!     ) -> ProviderFuture<'a, Result<Arc<dyn Greeter>, ProviderError>> {
//!         Box::pin(async {
//!             Ok(Arc::new(EnglishGreeter) as Arc<dyn Greeter>)
//!         })
//!     }
//! }
//!
//! impl ProviderMetadata for EnglishProvider {
//!     fn descriptor(&self) -> ProviderDescriptor {
//!         ProviderDescriptor::new(
//!             ProviderId::new("english").expect("static ID is valid"),
//!         )
//!     }
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = AsyncProviderRegistry::<GreeterSpec>::default();
//! registry.register(EnglishProvider)?;
//! let selection = ProviderSelection::named("english")?;
//! let resolver = registry.resolve_selected(&selection)?;
//! let greeter = resolver.create_configured(&()).await?;
//! assert_eq!("hello", greeter.greet());
//! # Ok(())
//! # }
//! ```

mod async_provider_definition;
mod async_provider_registry;
mod async_resolving_service_provider;
mod async_service_provider;
mod async_service_spec;
pub mod error;
mod fallback_policy;
mod internal;
mod missing_provider_policy;
mod provider_creation_termination;
mod provider_definition;
mod provider_descriptor;
mod provider_future;
mod provider_id;
mod provider_metadata;
mod provider_registry;
mod provider_selection;
mod provider_selection_target_ref;
mod provider_selector;
mod resolving_service_provider;
mod service_provider;
mod service_spec;
mod sync_service_spec;

pub use async_provider_definition::AsyncProviderDefinition;
pub use async_provider_registry::AsyncProviderRegistry;
pub use async_resolving_service_provider::AsyncResolvingServiceProvider;
pub use async_service_provider::AsyncServiceProvider;
pub use async_service_spec::AsyncServiceSpec;
pub use fallback_policy::FallbackPolicy;
pub use missing_provider_policy::MissingProviderPolicy;
pub use provider_creation_termination::ProviderCreationTermination;
pub use provider_definition::ProviderDefinition;
pub use provider_descriptor::ProviderDescriptor;
pub use provider_future::ProviderFuture;
pub use provider_id::ProviderId;
pub use provider_metadata::ProviderMetadata;
pub use provider_registry::ProviderRegistry;
pub use provider_selection::ProviderSelection;
pub use provider_selection_target_ref::ProviderSelectionTargetRef;
pub use provider_selector::ProviderSelector;
pub use resolving_service_provider::ResolvingServiceProvider;
pub use service_provider::ServiceProvider;
pub use service_spec::ServiceSpec;
pub use sync_service_spec::SyncServiceSpec;
