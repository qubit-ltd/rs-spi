// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed provider lookup results.

use crate::error::ProviderError;
use crate::internal::RegistryEntry;
use crate::{
    ProviderDescriptor,
    ServiceSpec,
};

/// Borrowed provider lookup result retaining descriptor metadata.
///
/// This value is returned by registry lookup and exposes the selected
/// provider's metadata and typed creation operation without exposing internals.
pub struct ResolvedProvider<'a, S>
where
    S: ServiceSpec,
{
    /// Internal entry borrowed from the immutable registry.
    entry: &'a RegistryEntry<S>,
}

impl<'a, S> ResolvedProvider<'a, S>
where
    S: ServiceSpec,
{
    /// Creates a lookup result borrowing one immutable registry entry.
    ///
    /// # Arguments
    ///
    /// * `entry` - Registry-owned entry selected by a validated index.
    ///
    /// # Returns
    ///
    /// A borrowed provider lookup result.
    #[inline]
    #[must_use]
    pub(crate) const fn new(entry: &'a RegistryEntry<S>) -> Self {
        Self { entry }
    }

    /// Returns the selected provider's registration metadata.
    ///
    /// # Returns
    ///
    /// The immutable descriptor stored with the resolved provider.
    #[inline(always)]
    #[must_use]
    pub fn descriptor(&self) -> &ProviderDescriptor {
        &self.entry.descriptor
    }

    /// Creates a service through the resolved provider factory.
    ///
    /// # Arguments
    ///
    /// * `config` - Service-family configuration forwarded unchanged.
    ///
    /// # Returns
    ///
    /// The provider's complete service output.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] when the provider cannot create the service;
    /// its classification remains available to resolver fallback logic.
    #[inline(always)]
    pub fn create(
        &self,
        config: &S::Config,
    ) -> Result<S::Output, ProviderError> {
        self.entry.provider.create(config)
    }
}
