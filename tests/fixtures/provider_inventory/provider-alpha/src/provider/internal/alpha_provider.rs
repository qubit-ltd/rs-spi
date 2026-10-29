// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Alpha provider implementation and inventory submission.

use std::convert::Infallible;

use service_contract::FixtureSpec;
use service_contract::sync_providers::Entry;
use spi::ProviderDescriptor;
use spi::ProviderMetadata;
use spi::ServiceProvider;
use spi::error::ProviderFailure;
use spi::provider_descriptor;
use spi::submit_sync_provider;

/// Implements the alpha fixture provider used by the inventory registration.
struct AlphaProvider;

impl ProviderMetadata for AlphaProvider {
    /// Returns alpha's stable registration descriptor.
    ///
    /// # Returns
    ///
    /// The descriptor used to identify this fixture provider as `alpha`.
    fn descriptor(&self) -> ProviderDescriptor {
        provider_descriptor!("alpha")
    }
}

impl ServiceProvider<FixtureSpec> for AlphaProvider {
    /// Creates the alpha fixture service output.
    ///
    /// # Parameters
    ///
    /// * `config` - Fixture configuration; this provider deliberately ignores it.
    ///
    /// # Returns
    ///
    /// The string `"alpha"` on success.
    ///
    /// # Errors
    ///
    /// This fixture implementation always succeeds and returns no error.
    fn create_configured(&self, _config: &String) -> Result<String, ProviderFailure<Infallible>> {
        Ok("alpha".to_owned())
    }
}

submit_sync_provider! {
    inventory_entry = Entry;
    spec = FixtureSpec;
    provider = AlphaProvider;
}
