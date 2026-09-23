// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Synchronous beta provider implementation and inventory submission.

use std::convert::Infallible;

use service_contract::FixtureSpec;
use service_contract::sync_providers::Entry;
use spi::ProviderDescriptor;
use spi::ProviderMetadata;
use spi::ServiceProvider;
use spi::error::ProviderFailure;
use spi::provider_descriptor;
use spi::submit_sync_provider;

/// Provider producing the beta fixture output.
struct BetaProvider;

impl ProviderMetadata for BetaProvider {
    /// Returns beta's stable registration descriptor.
    fn descriptor(&self) -> ProviderDescriptor {
        provider_descriptor!("beta")
    }
}

impl ServiceProvider<FixtureSpec> for BetaProvider {
    /// Creates the beta fixture service output.
    fn create_configured(&self, _config: &String) -> Result<String, ProviderFailure<Infallible>> {
        Ok("beta".to_owned())
    }
}

submit_sync_provider! {
    inventory_entry = Entry;
    spec = FixtureSpec;
    provider = BetaProvider;
}
