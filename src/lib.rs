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
//! use std::{
//!     error::Error,
//!     fmt,
//!     sync::Arc,
//! };
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
//! use qubit_spi::error::ProviderFailure;
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
//! #[derive(Debug)]
//! struct GreeterError;
//!
//! impl fmt::Display for GreeterError {
//!     fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         formatter.write_str("greeter creation failed")
//!     }
//! }
//!
//! impl Error for GreeterError {}
//!
//! impl ServiceSpec for GreeterSpec {
//!     type Config = ();
//!     type Error = GreeterError;
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
//!     ) -> Result<Arc<dyn Greeter>, ProviderFailure<GreeterError>> {
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
//! use std::{
//!     error::Error,
//!     fmt,
//!     sync::Arc,
//! };
//!
//! use qubit_spi::error::ProviderFailure;
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
//! #[derive(Debug)]
//! struct GreeterError;
//!
//! impl fmt::Display for GreeterError {
//!     fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         formatter.write_str("greeter creation failed")
//!     }
//! }
//!
//! impl Error for GreeterError {}
//!
//! impl ServiceSpec for GreeterSpec {
//!     type Config = ();
//!     type Error = GreeterError;
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
//!     ) -> ProviderFuture<'a, Result<Arc<dyn Greeter>, ProviderFailure<GreeterError>>> {
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

pub mod error;
mod provider;
mod registry;
mod selection;
mod service;

pub use provider::ProviderDescriptor;
pub use provider::ProviderId;
pub use provider::ProviderMetadata;
pub use provider::ProviderSelector;
pub use registry::AsyncProviderRegistry;
pub use registry::AsyncResolvingServiceProvider;
pub use registry::ProviderCreationTermination;
pub use registry::ProviderRegistry;
pub use registry::ResolvingServiceProvider;
pub use selection::FallbackPolicy;
pub use selection::MissingProviderPolicy;
pub use selection::ProviderSelection;
pub use selection::ProviderSelectionTargetRef;
pub use service::AsyncProviderDefinition;
pub use service::AsyncServiceProvider;
pub use service::AsyncServiceSpec;
pub use service::ProviderDefinition;
pub use service::ProviderFuture;
pub use service::ServiceProvider;
pub use service::ServiceSpec;
pub use service::SyncServiceSpec;
