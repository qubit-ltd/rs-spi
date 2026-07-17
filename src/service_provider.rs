// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider contract for pluggable service implementations.

use crate::ServiceSpec;
use crate::error::ProviderCreationError;

/// Factory contract for one pluggable service implementation.
///
/// This trait contains only service-creation behavior. Implement
/// [`crate::ProviderDefinition`] when the provider also needs to be registered
/// with identity and selection metadata. Implement this trait when an
/// application needs to provide one selectable backend for a [`ServiceSpec`]
/// family.
pub trait ServiceProvider<S>: Send + Sync + 'static
where
    S: ServiceSpec,
{
    /// Creates one service output from the supplied configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Service-family configuration declared by `S`.
    ///
    /// # Returns
    ///
    /// The complete `S::Output` handle.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderCreationError`] when the provider cannot create the
    /// requested service. A leaf provider should convert its classified
    /// [`crate::error::ProviderError`] with [`Into::into`]. A composing
    /// provider may instead return an aggregate creation error.
    fn create(
        &self,
        config: &S::Config,
    ) -> Result<S::Output, ProviderCreationError>;

    /// Creates one service output with the configuration default.
    ///
    /// # Returns
    ///
    /// The complete `S::Output` handle created from `S::Config::default()`.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderCreationError`] when the provider cannot create the
    /// service from the default configuration.
    #[inline]
    fn create_default(&self) -> Result<S::Output, ProviderCreationError>
    where
        S::Config: Default,
    {
        self.create(&S::Config::default())
    }
}
