// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised when mutating a provider registry.

use thiserror::Error;

/// Error returned when a registry mutation conflicts with its current state.
///
/// Duplicate selector errors retain the selector and both provider IDs so an
/// application can report the registration conflict. A sealed registry
/// rejects every later mutation.
///
/// # Examples
///
/// ```rust
/// use qubit_spi::{
///     ProviderDescriptor, ProviderId, ProviderMetadata, ProviderRegistry,
///     ServiceProvider, ServiceSpec, SyncServiceSpec,
/// };
/// use qubit_spi::error::{ProviderFailure, RegistryMutationError};
///
/// struct Spec;
/// impl ServiceSpec for Spec {
///     type Config = ();
///     type Error = std::io::Error;
/// }
/// impl SyncServiceSpec for Spec { type Output = (); }
/// struct Backend;
/// impl ProviderMetadata for Backend {
///     fn descriptor(&self) -> ProviderDescriptor {
///         ProviderDescriptor::new(ProviderId::new("backend").expect("valid ID"))
///     }
/// }
/// impl ServiceProvider<Spec> for Backend {
///     fn create_configured(&self, _: &()) -> Result<(), ProviderFailure<std::io::Error>> {
///         Ok(())
///     }
/// }
/// let registry = ProviderRegistry::<Spec>::default();
/// registry.register(Backend)?;
/// let error = registry.register(Backend).expect_err("ID is already registered");
/// assert!(matches!(error, RegistryMutationError::DuplicateSelector { .. }));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
#[must_use]
pub enum RegistryMutationError {
    /// A selector is already owned by another provider.
    #[error("provider selector {selector} claimed by {provider} is already owned by {existing_provider}")]
    DuplicateSelector {
        /// Selector that has already been claimed.
        selector: Box<str>,
        /// Identifier of the provider that already owns the selector.
        existing_provider: Box<str>,
        /// Identifier of the provider that attempted to claim the selector.
        provider: Box<str>,
    },
    /// The registry has been sealed and cannot be mutated.
    #[error("provider registry is sealed")]
    Sealed,
}

impl RegistryMutationError {
    /// Creates a duplicate-selector error with both provider identities.
    ///
    /// # Parameters
    ///
    /// * `selector` - Normalized selector already claimed in the registry.
    /// * `existing_provider` - Canonical ID that currently owns the selector.
    /// * `provider` - Canonical ID attempting to claim the selector.
    ///
    /// # Returns
    ///
    /// A mutation error retaining the conflicting registration details.
    pub(crate) fn duplicate_selector(selector: &str, existing_provider: &str, provider: &str) -> Self {
        Self::DuplicateSelector {
            selector: selector.into(),
            existing_provider: existing_provider.into(),
            provider: provider.into(),
        }
    }

    /// Returns the duplicate selector, or [`None`] when the registry is sealed.
    #[must_use]
    pub fn selector(&self) -> Option<&str> {
        match self {
            Self::DuplicateSelector { selector, .. } => Some(selector),
            Self::Sealed => None,
        }
    }

    /// Returns the provider that already owns the selector, or [`None`] when
    /// the registry is sealed.
    #[must_use]
    pub fn existing_provider(&self) -> Option<&str> {
        match self {
            Self::DuplicateSelector { existing_provider, .. } => Some(existing_provider),
            Self::Sealed => None,
        }
    }

    /// Returns the provider that attempted to claim the selector, or [`None`]
    /// when the registry is sealed.
    #[must_use]
    pub fn provider(&self) -> Option<&str> {
        match self {
            Self::DuplicateSelector { provider, .. } => Some(provider),
            Self::Sealed => None,
        }
    }

    /// Returns `true` when the registry has been sealed and rejects mutations.
    #[must_use]
    pub const fn is_sealed(&self) -> bool {
        matches!(self, Self::Sealed)
    }
}
