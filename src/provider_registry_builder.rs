// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Optional fluent assembly of provider registries.

use std::sync::Arc;

use crate::error::RegistrationError;
use crate::{
    ProviderDefinition,
    ProviderRegistry,
    ServiceSpec,
};

/// Optional mutable wrapper for assembling a provider registry.
///
/// Use this type when fluent startup assembly is convenient. The resulting
/// [`ProviderRegistry`] remains open to later runtime registrations.
pub struct ProviderRegistryBuilder<S>
where
    S: ServiceSpec,
{
    /// Registry receiving each validated registration immediately.
    registry: ProviderRegistry<S>,
}

impl<S> ProviderRegistryBuilder<S>
where
    S: ServiceSpec,
{
    /// Creates an empty provider-registry builder.
    ///
    /// # Returns
    ///
    /// A builder containing an empty runtime registry.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: ProviderRegistry::default(),
        }
    }

    /// Registers an owned provider factory.
    ///
    /// # Arguments
    ///
    /// * `provider` - Self-described factory moved into shared registry
    ///   storage.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the registration and selector claims are recorded.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when the provider's descriptor conflicts
    /// with an earlier registration.
    #[inline(always)]
    pub fn register<P>(&mut self, provider: P) -> Result<(), RegistrationError>
    where
        P: ProviderDefinition<S>,
    {
        self.register_shared(Arc::new(provider))
    }

    /// Registers an already shared provider factory.
    ///
    /// # Arguments
    ///
    /// * `provider` - Shared self-described factory retained by the runtime
    ///   registry.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the registration and selector claims are recorded.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when the provider's descriptor conflicts
    /// with an earlier registration.
    #[inline(always)]
    pub fn register_shared(
        &mut self,
        provider: Arc<dyn ProviderDefinition<S>>,
    ) -> Result<(), RegistrationError> {
        self.registry.register_shared(provider)
    }

    /// Returns the assembled runtime provider registry.
    ///
    /// # Returns
    ///
    /// The registry that received every successful builder registration.
    #[must_use]
    pub fn build(self) -> ProviderRegistry<S> {
        self.registry
    }
}

impl<S> Default for ProviderRegistryBuilder<S>
where
    S: ServiceSpec,
{
    /// Creates an empty provider-registry builder.
    ///
    /// # Returns
    ///
    /// A builder with no registrations or claimed selectors.
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}
