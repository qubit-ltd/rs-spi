// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider contract for pluggable service implementations.

use crate::SyncServiceSpec;
use crate::error::ProviderError;

/// Factory contract for one pluggable service implementation.
///
/// This trait contains only service-creation behavior. Implement
/// [`crate::ProviderDefinition`] when the provider also needs to be registered
/// with identity and selection metadata. Implement this trait when an
/// application needs to provide one selectable backend for a
/// [`SyncServiceSpec`]
/// family. Registry fallback applies only to errors returned while these
/// creation methods run. Once a service output is returned successfully,
/// errors from later operations on that output do not trigger another
/// provider attempt.
pub trait ServiceProvider<S>: Send + Sync + 'static
where
    S: SyncServiceSpec,
{
    /// Creates one service output from the supplied configuration.
    ///
    /// # Parameters
    ///
    /// * `config` - Service-family configuration declared by `S`.
    ///
    /// # Returns
    ///
    /// The complete `S::Output` handle.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] when this provider cannot create the requested
    /// service. Registry resolvers aggregate this leaf failure with provider
    /// identity and fallback termination diagnostics.
    fn create_configured(
        &self,
        config: &S::Config,
    ) -> Result<S::Output, ProviderError>;

    /// Creates one service output with the configuration default.
    ///
    /// # Returns
    ///
    /// The complete `S::Output` handle created from `S::Config::default()`.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] when the provider cannot create the service
    /// from the default configuration.
    #[inline(always)]
    fn create(&self) -> Result<S::Output, ProviderError>
    where
        S::Config: Default,
    {
        self.create_configured(&S::Config::default())
    }
}
