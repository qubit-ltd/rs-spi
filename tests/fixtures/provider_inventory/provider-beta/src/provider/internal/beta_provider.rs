// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Beta provider implementation and inventory submission.

use std::convert::Infallible;

use service_contract::FixtureSpec;
use service_contract::sync_providers::Entry;
use spi::ProviderDescriptor;
use spi::ProviderMetadata;
use spi::ServiceProvider;
use spi::error::ProviderFailure;
use spi::provider_descriptor;
use spi::submit_sync_provider;

/// Produces the beta fixture output and registers it for the fixture service family.
///
/// The provider is private to this fixture crate and is constructed by the inventory
/// registration generated below.
struct BetaProvider;

impl ProviderMetadata for BetaProvider {
    /// Returns beta's stable registration descriptor.
    ///
    /// # Returns
    ///
    /// The descriptor identifies this provider with the stable `beta` ID.
    fn descriptor(&self) -> ProviderDescriptor {
        provider_descriptor!("beta")
    }
}

impl ServiceProvider<FixtureSpec> for BetaProvider {
    /// Creates the beta fixture service output.
    ///
    /// # Parameters
    ///
    /// * `config` - Fixture configuration, which this provider intentionally ignores.
    ///
    /// # Returns
    ///
    /// The constant beta fixture output.
    ///
    /// # Errors
    ///
    /// This implementation always succeeds and does not construct a provider failure.
    fn create_configured(&self, _config: &String) -> Result<String, ProviderFailure<Infallible>> {
        Ok("beta".to_owned())
    }
}

submit_sync_provider! {
    inventory_entry = Entry;
    spec = FixtureSpec;
    provider = BetaProvider;
}
