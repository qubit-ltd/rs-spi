// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous provider contract for pluggable service implementations.

use crate::AsyncServiceSpec;
use crate::ProviderFuture;
use crate::error::ProviderFailure;

/// Runtime-independent asynchronous factory contract for one provider.
///
/// # Type Parameters
///
/// * `S` - Asynchronous service family created by this provider.
pub trait AsyncServiceProvider<S>: Send + Sync + 'static
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Creates one service output with the default configuration.
    ///
    /// # Returns
    ///
    /// A sendable future yielding the service output or leaf provider failure.
    ///
    /// # Errors
    ///
    /// The future yields ProviderFailure when the provider cannot create the
    /// service from the default configuration.
    ///
    /// # Panics
    ///
    /// The returned future may panic if `S::Config::default()` or
    /// [`Self::create_configured`] panics.
    fn create(&self) -> ProviderFuture<'_, Result<S::Output, ProviderFailure<S::Error>>>
    where
        S::Config: Default + Send,
    {
        Box::pin(async move {
            let config = S::Config::default();
            self.create_configured(&config).await
        })
    }

    /// Creates one service output from the supplied configuration.
    ///
    /// # Type Parameters
    ///
    /// * `'a` - Lifetime shared by the provider, configuration, and future.
    ///
    /// # Parameters
    ///
    /// * `config` - Service-family configuration borrowed by the future.
    ///
    /// # Returns
    ///
    /// A sendable future yielding the complete service output or one
    /// classified leaf provider failure.
    ///
    /// # Errors
    ///
    /// The future yields ProviderFailure when this provider cannot create
    /// the requested service.
    ///
    /// # Panics
    ///
    /// The returned future may panic if the provider implementation or its
    /// creation future panics.
    fn create_configured<'a>(
        &'a self,
        config: &'a S::Config,
    ) -> ProviderFuture<'a, Result<S::Output, ProviderFailure<S::Error>>>;
}
