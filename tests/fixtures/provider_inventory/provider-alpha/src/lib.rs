// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Synchronous alpha provider submitted to the fixture contract inventory.

use std::convert::Infallible;

use spi::ProviderDescriptor;
use spi::ProviderMetadata;
use spi::ServiceProvider;
use spi::error::ProviderFailure;

/// Provider producing the alpha fixture output.
struct AlphaProvider;

impl ProviderMetadata for AlphaProvider {
    /// Returns alpha's stable registration descriptor.
    fn descriptor(&self) -> ProviderDescriptor {
        spi::provider_descriptor!("alpha")
    }
}

impl ServiceProvider<service_contract::FixtureSpec> for AlphaProvider {
    /// Creates the alpha fixture service output.
    fn create_configured(&self, _config: &String) -> Result<String, ProviderFailure<Infallible>> {
        Ok("alpha".to_owned())
    }
}

spi::submit_sync_provider! {
    inventory_entry = ::service_contract::sync_providers::Entry;
    spec = service_contract::FixtureSpec;
    provider = AlphaProvider;
}
