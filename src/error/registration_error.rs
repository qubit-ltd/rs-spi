// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Conflicts raised while registering providers.

use thiserror::Error;

/// Error returned when a provider registration conflicts with registry state.
///
/// # Examples
///
/// ```rust
/// use qubit_spi::{ServiceSpec, SyncServiceSpec};
/// struct Spec;
/// impl ServiceSpec for Spec {
///     type Config = String;
///     type Error = std::io::Error;
/// }
/// impl SyncServiceSpec for Spec { type Output = String; }
/// use qubit_spi::{ProviderDescriptor, ProviderId, ProviderMetadata, ServiceProvider};
/// use qubit_spi::error::ProviderFailure;
/// struct Echo;
/// impl ProviderMetadata for Echo {
///     fn descriptor(&self) -> ProviderDescriptor {
///         ProviderDescriptor::new(ProviderId::new("echo").expect("valid static ID"))
///     }
/// }
/// impl ServiceProvider<Spec> for Echo {
///     fn create_configured(&self, config: &String) -> Result<String, ProviderFailure<std::io::Error>> {
///         Ok(config.clone())
///     }
/// }
/// let registry = qubit_spi::ProviderRegistry::<Spec>::default();
/// registry.register(Echo)?;
/// let error = registry.register(Echo).expect_err("canonical ID already belongs to a provider");
/// assert!(matches!(error, qubit_spi::error::RegistrationError::DuplicateSelector { .. }));
/// assert_eq!(1, registry.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
#[must_use]
pub enum RegistrationError {
    /// A selector is already owned by a registered provider.
    #[non_exhaustive]
    #[error("provider selector {selector} claimed by {provider} is already owned by {existing_provider}")]
    DuplicateSelector {
        /// Conflicting canonical ID or alias.
        selector: Box<str>,
        /// Canonical ID that already owns the selector.
        existing_provider: Box<str>,
        /// Canonical ID attempting the new claim.
        provider: Box<str>,
    },
}

impl RegistrationError {
    /// Creates an error for a selector claimed by another provider.
    ///
    /// # Parameters
    ///
    /// * `selector` - Conflicting canonical ID or alias.
    /// * `existing_provider` - Canonical provider currently owning the
    ///   selector.
    /// * `provider` - Canonical provider attempting the new claim.
    ///
    /// # Returns
    ///
    /// A registry-owned duplicate-selector error.
    #[inline]
    pub(crate) fn duplicate_selector(selector: &str, existing_provider: &str, provider: &str) -> Self {
        Self::DuplicateSelector {
            selector: selector.into(),
            existing_provider: existing_provider.into(),
            provider: provider.into(),
        }
    }

    /// Returns the canonical ID or alias that caused this conflict.
    ///
    /// # Returns
    ///
    /// The selector claimed by both providers.
    #[inline(always)]
    #[must_use]
    pub fn selector(&self) -> &str {
        match self {
            Self::DuplicateSelector { selector, .. } => selector,
        }
    }

    /// Returns the canonical ID of the provider that owns the selector.
    ///
    /// # Returns
    ///
    /// The registered provider that already owns the conflicting selector.
    #[inline(always)]
    #[must_use]
    pub fn existing_provider(&self) -> &str {
        match self {
            Self::DuplicateSelector { existing_provider, .. } => existing_provider,
        }
    }

    /// Returns the canonical ID of the provider attempting registration.
    ///
    /// # Returns
    ///
    /// The provider whose registration conflicts with existing registry state.
    #[inline(always)]
    #[must_use]
    pub fn provider(&self) -> &str {
        match self {
            Self::DuplicateSelector { provider, .. } => provider,
        }
    }
}
