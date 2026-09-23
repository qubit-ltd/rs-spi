// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous provider submitted to the fixture contract inventory.

use std::convert::Infallible;

use spi::AsyncServiceProvider;
use spi::ProviderDescriptor;
use spi::ProviderFuture;
use spi::ProviderMetadata;
use spi::error::ProviderFailure;

/// Provider producing the asynchronous fixture output.
struct AsyncProvider;

impl ProviderMetadata for AsyncProvider {
    /// Returns the asynchronous provider's stable registration descriptor.
    fn descriptor(&self) -> ProviderDescriptor {
        spi::provider_descriptor!("async")
    }
}

impl AsyncServiceProvider<service_contract::FixtureSpec> for AsyncProvider {
    /// Returns a future that creates the asynchronous fixture output.
    fn create_configured<'a>(
        &'a self,
        _config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<Infallible>>> {
        Box::pin(async { Ok("async".to_owned()) })
    }
}

spi::submit_async_provider! {
    inventory_entry = ::service_contract::async_providers::Entry;
    spec = service_contract::FixtureSpec;
    provider = AsyncProvider;
}
