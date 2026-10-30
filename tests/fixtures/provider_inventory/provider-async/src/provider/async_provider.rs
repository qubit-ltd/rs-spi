// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fixed asynchronous provider implementation.

use std::convert::Infallible;

use service_contract::FixtureSpec;
use spi::AsyncServiceProvider;
use spi::ProviderDescriptor;
use spi::ProviderFuture;
use spi::ProviderMetadata;
use spi::error::ProviderFailure;
use spi::provider_descriptor;

/// Supplies the fixed output used by the asynchronous inventory fixture.
pub(super) struct AsyncProvider;

impl ProviderMetadata for AsyncProvider {
    /// Describes the asynchronous fixture provider for inventory registration.
    ///
    /// # Returns
    ///
    /// A descriptor with the fixture's stable `async` identifier.
    fn descriptor(&self) -> ProviderDescriptor {
        provider_descriptor!("async")
    }
}

impl AsyncServiceProvider<FixtureSpec> for AsyncProvider {
    /// Creates the fixed asynchronous fixture output.
    ///
    /// # Type Parameters
    ///
    /// * `'a` - Lifetime shared by the provider, configuration, and future.
    ///
    /// # Parameters
    ///
    /// * `_config` - Fixture configuration, which does not affect the fixed
    ///   output.
    ///
    /// # Returns
    ///
    /// A future that resolves to the literal `async` output.
    ///
    /// # Errors
    ///
    /// This fixture cannot produce a provider failure because its error type
    /// is `Infallible`.
    fn create_configured<'a>(
        &'a self,
        _config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<Infallible>>> {
        Box::pin(async { Ok("async".to_owned()) })
    }
}
