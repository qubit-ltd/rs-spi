// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous provider implementation and inventory submission.

use std::convert::Infallible;

use service_contract::FixtureSpec;
use service_contract::async_providers::Entry;
use spi::AsyncServiceProvider;
use spi::ProviderDescriptor;
use spi::ProviderFuture;
use spi::ProviderMetadata;
use spi::error::ProviderFailure;
use spi::provider_descriptor;
use spi::submit_async_provider;

/// Provider producing the asynchronous fixture output.
struct AsyncProvider;

impl ProviderMetadata for AsyncProvider {
    /// Returns the asynchronous provider's stable registration descriptor.
    fn descriptor(&self) -> ProviderDescriptor {
        provider_descriptor!("async")
    }
}

impl AsyncServiceProvider<FixtureSpec> for AsyncProvider {
    /// Returns a future that creates the asynchronous fixture output.
    fn create_configured<'a>(
        &'a self,
        _config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<Infallible>>> {
        Box::pin(async { Ok("async".to_owned()) })
    }
}

submit_async_provider! {
    inventory_entry = Entry;
    spec = FixtureSpec;
    provider = AsyncProvider;
}
