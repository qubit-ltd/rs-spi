//! # Qubit SPI
//!
//! Typed service provider infrastructure for Qubit Rust crates.
//!
//! This crate provides the small service-provider layer that many Qubit crates
//! need when a base crate owns a trait and extension crates provide optional
//! implementations. A [`ServiceProvider`] supplies stable names, aliases,
//! runtime availability, priority, and a factory method. A
//! [`ProviderRegistry`] resolves providers by name, by automatic priority, or
//! through an explicit fallback chain.
//!
//! # Examples
//!
//! Register a provider and create a service by name:
//!
//! ```rust
//! use std::fmt::Debug;
//!
//! use qubit_spi::{
//!     ProviderCreateError,
//!     ProviderDescriptor,
//!     ProviderRegistry,
//!     ProviderRegistryError,
//!     ServiceProvider,
//!     ServiceSpec,
//! };
//!
//! trait Greeter: Debug + Send + Sync {
//!     fn greet(&self) -> &'static str;
//! }
//!
//! #[derive(Debug)]
//! struct EnglishGreeter;
//!
//! impl Greeter for EnglishGreeter {
//!     fn greet(&self) -> &'static str {
//!         "hello"
//!     }
//! }
//!
//! #[derive(Debug)]
//! struct EnglishProvider;
//!
//! #[derive(Debug)]
//! struct GreeterSpec;
//!
//! impl ServiceSpec for GreeterSpec {
//!     type Config = ();
//!     type Service = dyn Greeter;
//! }
//!
//! impl ServiceProvider<GreeterSpec> for EnglishProvider {
//!     fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
//!         ProviderDescriptor::new("english")?.with_aliases(&["en"])
//!     }
//!
//!     fn create_box(&self, _config: &()) -> Result<Box<dyn Greeter>, ProviderCreateError> {
//!         Ok(Box::new(EnglishGreeter))
//!     }
//! }
//!
//! let mut registry = ProviderRegistry::<GreeterSpec>::new();
//! registry
//!     .register(EnglishProvider)
//!     .expect("provider names should be unique");
//!
//! let greeter = registry
//!     .create_box("en", &())
//!     .expect("registered provider should create a greeter");
//! assert_eq!("hello", greeter.greet());
//! ```

mod provider_availability;
mod provider_create_error;
mod provider_descriptor;
mod provider_failure;
mod provider_name;
mod provider_registry;
mod provider_registry_error;
mod provider_selection;
mod service_provider;
mod service_spec;

pub use provider_availability::ProviderAvailability;
pub use provider_create_error::ProviderCreateError;
pub use provider_descriptor::ProviderDescriptor;
pub use provider_failure::ProviderFailure;
pub use provider_name::ProviderName;
pub use provider_registry::ProviderRegistry;
pub use provider_registry_error::ProviderRegistryError;
pub use provider_selection::ProviderSelection;
pub use service_provider::ServiceProvider;
pub use service_spec::ServiceSpec;
