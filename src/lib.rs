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
//! ~~~rust
//! use std::sync::Arc;
//!
//! use qubit_spi::{
//!     FallbackPolicy,
//!     ProviderDescriptor,
//!     ProviderId,
//!     ProviderRegistry,
//!     ProviderResolver,
//!     ProviderSelection,
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
//!     ) -> Result<Arc<dyn Greeter>, qubit_spi::ProviderError> {
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
//! let created = resolver.create(&ProviderSelection::named("en")?, &())?;
//! assert_eq!("hello", created.service().greet());
//! # Ok(())
//! # }
//! ~~~

mod created_service;
mod provider_descriptor;
mod provider_error;
mod provider_id;
mod provider_registration;
mod provider_registry;
mod provider_registry_builder;
mod provider_resolver;
mod provider_selection;
mod provider_selector;
mod registration_error;
mod resolution_error;
mod service_provider;
mod service_spec;

pub use created_service::CreatedService;
pub use provider_descriptor::ProviderDescriptor;
pub use provider_error::{ProviderError, ProviderErrorKind};
pub use provider_id::ProviderId;
pub use provider_registration::ProviderRegistration;
pub use provider_registry::{ProviderRegistry, ResolvedProvider};
pub use provider_registry_builder::ProviderRegistryBuilder;
pub use provider_resolver::ProviderResolver;
pub use provider_selection::{FallbackPolicy, ProviderSelection};
pub use provider_selector::ProviderSelector;
pub use registration_error::{RegistrationError, RegistrationErrorKind};
pub use resolution_error::{AttemptFailure, ResolutionError, ResolutionErrorKind};
pub use service_provider::ServiceProvider;
pub use service_spec::ServiceSpec;
