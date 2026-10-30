// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider implementation used to exercise duplicate inventory rejection.

use std::convert::Infallible;

use spi::ProviderDescriptor;
use spi::ProviderMetadata;
use spi::ServiceProvider;
use spi::error::ProviderFailure;
use spi::provider_descriptor;

use super::super::FixtureSpec;

/// Provider that conflicts with alpha during registry construction.
///
/// This fixture registers the same provider key as the alpha fixture so the
/// inventory tests can verify duplicate-key rejection.
pub(super) struct DuplicateProvider;

impl ProviderMetadata for DuplicateProvider {
    /// Returns the duplicate alpha registration descriptor.
    fn descriptor(&self) -> ProviderDescriptor {
        provider_descriptor!("alpha")
    }
}

impl ServiceProvider<FixtureSpec> for DuplicateProvider {
    /// Creates an unreachable fixture output because registration must fail
    /// first.
    fn create_configured(
        &self,
        _config: &String,
    ) -> Result<String, ProviderFailure<Infallible>> {
        Ok("duplicate".to_owned())
    }
}
