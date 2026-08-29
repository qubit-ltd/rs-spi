// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::AsyncProviderDefinition;
use qubit_spi::AsyncServiceProvider;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderFuture;
use qubit_spi::ProviderId;
use qubit_spi::ProviderMetadata;
use qubit_spi::error::ProviderFailure;

use crate::common::async_configurable_provider::AsyncConfigurableProvider;
use crate::common::string_spec::StringSpec;

/// Metadata-bearing asynchronous provider fixture.
struct DescribedAsyncProvider {
    descriptor: ProviderDescriptor,
    provider: AsyncConfigurableProvider,
}

impl ProviderMetadata for DescribedAsyncProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }
}

impl AsyncServiceProvider<StringSpec> for DescribedAsyncProvider {
    fn create_configured<'a>(
        &'a self,
        config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<crate::common::test_error::TestError>>> {
        self.provider.create_configured(config)
    }
}

/// Verifies the asynchronous definition blanket implementation.
#[test]
fn test_metadata_and_async_provider_form_definition_automatically() {
    fn assert_definition<T: AsyncProviderDefinition<StringSpec>>() {}

    assert_definition::<DescribedAsyncProvider>();
    let provider = DescribedAsyncProvider {
        descriptor: ProviderDescriptor::new(ProviderId::new("async").expect("test provider ID should be valid")),
        provider: AsyncConfigurableProvider::success("output"),
    };
    assert_eq!("async", provider.descriptor().id().as_str());
}
