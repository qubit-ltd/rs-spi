// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::ProviderCreationError;
use qubit_spi::{
    ProviderDefinition,
    ProviderDescriptor,
    ServiceProvider,
    ServiceSpec,
};

/// A self-described provider assembled from existing test fixtures.
pub(crate) struct TestProviderDefinition<P> {
    descriptor: ProviderDescriptor,
    provider: P,
}

impl<S, P> ServiceProvider<S> for TestProviderDefinition<P>
where
    S: ServiceSpec,
    P: ServiceProvider<S>,
{
    fn create(
        &self,
        config: &S::Config,
    ) -> Result<S::Output, ProviderCreationError> {
        self.provider.create(config)
    }
}

impl<S, P> ProviderDefinition<S> for TestProviderDefinition<P>
where
    S: ServiceSpec,
    P: ServiceProvider<S>,
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
