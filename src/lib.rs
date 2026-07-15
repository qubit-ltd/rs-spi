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
//! let created = resolver.create_named("en", &())?;
//! assert_eq!("hello", created.service().greet());
//! # Ok(())
//! # }
//! ```

mod attempt_failure;
mod attempt_failure_kind;
mod created_service;
mod fallback_policy;
mod internal;
mod provider_descriptor;
mod provider_descriptor_error;
mod provider_descriptor_error_kind;
mod provider_error;
mod provider_error_kind;
mod provider_id;
mod provider_id_error;
mod provider_id_error_kind;
mod provider_registry;
mod provider_registry_builder;
mod provider_resolver;
mod provider_selection;
mod provider_selection_error;
mod provider_selection_error_kind;
mod provider_selection_kind;
mod provider_selector;
mod provider_selector_error;
mod provider_selector_error_kind;
mod registration_error;
mod registration_error_kind;
mod resolution_error;
mod resolution_error_kind;
mod resolved_provider;
mod service_provider;
mod service_spec;

pub use attempt_failure::AttemptFailure;
pub use attempt_failure_kind::AttemptFailureKind;
pub use created_service::CreatedService;
pub use fallback_policy::FallbackPolicy;
pub use provider_descriptor::ProviderDescriptor;
pub use provider_descriptor_error::ProviderDescriptorError;
pub use provider_descriptor_error_kind::ProviderDescriptorErrorKind;
pub use provider_error::ProviderError;
pub use provider_error_kind::ProviderErrorKind;
pub use provider_id::ProviderId;
pub use provider_id_error::ProviderIdError;
pub use provider_id_error_kind::ProviderIdErrorKind;
pub use provider_registry::ProviderRegistry;
pub use provider_registry_builder::ProviderRegistryBuilder;
pub use provider_resolver::ProviderResolver;
pub use provider_selection::ProviderSelection;
pub use provider_selection_error::ProviderSelectionError;
pub use provider_selection_error_kind::ProviderSelectionErrorKind;
pub use provider_selection_kind::ProviderSelectionKind;
pub use provider_selector::ProviderSelector;
pub use provider_selector_error::ProviderSelectorError;
pub use provider_selector_error_kind::ProviderSelectorErrorKind;
pub use registration_error::RegistrationError;
pub use registration_error_kind::RegistrationErrorKind;
pub use resolution_error::ResolutionError;
pub use resolution_error_kind::ResolutionErrorKind;
pub use resolved_provider::ResolvedProvider;
pub use service_provider::ServiceProvider;
pub use service_spec::ServiceSpec;
