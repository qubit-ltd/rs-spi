// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Alpha provider implementation registered in the fixture inventory.

use std::convert::Infallible;

use service_contract::FixtureSpec;
use service_contract::sync_providers::Entry;
use spi::ProviderDescriptor;
use spi::ProviderMetadata;
use spi::ServiceProvider;
use spi::error::ProviderFailure;
use spi::provider_descriptor;
use spi::submit_sync_provider;

/// Supplies the fixed alpha implementation used by the inventory fixture.
///
/// The provider ignores service configuration and always creates the `alpha`
/// output so inventory registration can be exercised across crate boundaries.
struct AlphaProvider;

impl ProviderMetadata for AlphaProvider {
    /// Returns alpha's stable registration descriptor.
    ///
    /// # Returns
    ///
    /// The descriptor uses `alpha` as its canonical provider ID.
    fn descriptor(&self) -> ProviderDescriptor {
        provider_descriptor!("alpha")
    }
}

impl ServiceProvider<FixtureSpec> for AlphaProvider {
    /// Creates the alpha fixture service output.
    ///
    /// # Parameters
    ///
    /// * `_config` - Configuration supplied by the service family; this fixture
    ///   intentionally ignores it.
    ///
    /// # Returns
    ///
    /// The fixed `alpha` output.
    ///
    /// # Errors
    ///
    /// This implementation cannot fail because the fixture's domain error is
    /// [`Infallible`].
    fn create_configured(
        &self,
        _config: &String,
    ) -> Result<String, ProviderFailure<Infallible>> {
        Ok("alpha".to_owned())
    }
}

submit_sync_provider! {
    inventory_entry = Entry;
    spec = FixtureSpec;
    provider = AlphaProvider;
}
