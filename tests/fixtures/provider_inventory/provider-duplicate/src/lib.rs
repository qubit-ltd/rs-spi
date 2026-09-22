// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider intentionally colliding with alpha's canonical ID.

use std::convert::Infallible;

use spi::ProviderDescriptor;
use spi::ProviderMetadata;
use spi::ServiceProvider;
use spi::error::ProviderFailure;

/// Provider that conflicts with alpha during registry construction.
struct DuplicateProvider;

impl ProviderMetadata for DuplicateProvider {
    /// Returns the duplicate alpha registration descriptor.
    fn descriptor(&self) -> ProviderDescriptor {
        spi::provider_descriptor!("alpha")
    }
}

impl ServiceProvider<service_contract::FixtureSpec> for DuplicateProvider {
    /// Creates an unreachable fixture output because registration must fail
    /// first.
    fn create_configured(&self, _config: &String) -> Result<String, ProviderFailure<Infallible>> {
        Ok("duplicate".to_owned())
    }
}

spi::submit_sync_provider! {
    inventory_entry = service_contract::sync_providers::Entry;
    spec = service_contract::FixtureSpec;
    provider = DuplicateProvider;
}
