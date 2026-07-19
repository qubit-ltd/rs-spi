// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous provider contract for pluggable service implementations.

use crate::error::ProviderError;
use crate::{
    AsyncServiceSpec,
    ProviderFuture,
};

/// Runtime-independent asynchronous factory contract for one provider.
pub trait AsyncServiceProvider<S>: Send + Sync + 'static
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Creates one service output from the supplied configuration.
    ///
    /// # Parameters
    ///
    /// * `config` - Service-family configuration borrowed by the future.
    ///
    /// # Returns
    ///
    /// A sendable future yielding the complete service output or one
    /// classified leaf provider failure.
    fn create_configured<'a>(
        &'a self,
        config: &'a S::Config,
    ) -> ProviderFuture<'a, Result<S::Output, ProviderError>>;

    /// Creates one service output with the default configuration.
    ///
    /// # Returns
    ///
    /// A sendable future yielding the service output or leaf provider failure.
    fn create(&self) -> ProviderFuture<'_, Result<S::Output, ProviderError>>
    where
        S::Config: Default + Send,
    {
        Box::pin(async move {
            let config = S::Config::default();
            self.create_configured(&config).await
        })
    }
}
