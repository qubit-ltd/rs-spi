// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::AsyncServiceProvider;
use qubit_spi::AsyncServiceSpec;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderFuture;
use qubit_spi::ProviderMetadata;
use qubit_spi::ServiceProvider;
use qubit_spi::SyncServiceSpec;
use qubit_spi::error::ProviderFailure;

/// A self-described provider assembled from existing test fixtures.
pub(crate) struct TestProviderDefinition<P> {
    descriptor: ProviderDescriptor,
    provider: P,
}

impl<S, P> AsyncServiceProvider<S> for TestProviderDefinition<P>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
    P: AsyncServiceProvider<S>,
{
    fn create_configured<'a>(
        &'a self,
        config: &'a S::Config,
    ) -> ProviderFuture<'a, Result<S::Output, ProviderFailure<S::Error>>> {
        self.provider.create_configured(config)
    }
}

impl<S, P> ServiceProvider<S> for TestProviderDefinition<P>
where
    S: SyncServiceSpec,
    P: ServiceProvider<S>,
{
    fn create_configured(
        &self,
        config: &S::Config,
    ) -> Result<S::Output, ProviderFailure<S::Error>> {
        self.provider.create_configured(config)
    }
}

impl<P> ProviderMetadata for TestProviderDefinition<P>
where
    P: Send + Sync + 'static,
{
    fn descriptor(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }
}

/// Wraps a provider fixture with its registration descriptor.
pub(crate) fn define_provider<P>(
    descriptor: ProviderDescriptor,
    provider: P,
) -> TestProviderDefinition<P> {
    TestProviderDefinition {
        descriptor,
        provider,
    }
}
