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
//!     ProviderRegistry,
//!     ProviderRegistryError,
//!     ServiceProvider,
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
//! impl ServiceProvider for EnglishProvider {
//!     type Config = ();
//!     type Service = dyn Greeter;
//!
//!     fn id(&self) -> &'static str {
//!         "english"
//!     }
//!
//!     fn aliases(&self) -> &'static [&'static str] {
//!         &["en"]
//!     }
//!
//!     fn create(&self, _config: &Self::Config) -> Result<Box<Self::Service>, ProviderRegistryError> {
//!         Ok(Box::new(EnglishGreeter))
//!     }
//! }
//!
//! let mut registry = ProviderRegistry::<dyn Greeter, ()>::new();
//! registry
//!     .register(EnglishProvider)
//!     .expect("provider names should be unique");
//!
//! let greeter = registry
//!     .create("en", &())
//!     .expect("registered provider should create a greeter");
//! assert_eq!("hello", greeter.greet());
//! ```

mod provider_availability;
mod provider_failure;
mod provider_registry;
mod provider_registry_error;
mod provider_selection;
mod service_provider;

pub use provider_availability::ProviderAvailability;
pub use provider_failure::ProviderFailure;
pub use provider_registry::ProviderRegistry;
pub use provider_registry_error::ProviderRegistryError;
pub use provider_selection::ProviderSelection;
pub use service_provider::ServiceProvider;
