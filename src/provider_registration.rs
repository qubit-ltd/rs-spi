// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! A provider factory paired with immutable registration metadata.

use std::sync::Arc;

use crate::{ProviderDescriptor, ServiceProvider, ServiceSpec};

/// One explicit provider registration.
pub struct ProviderRegistration<S>
where
    S: ServiceSpec,
{
    descriptor: ProviderDescriptor,
    provider: Arc<dyn ServiceProvider<S>>,
}

impl<S> ProviderRegistration<S>
where
    S: ServiceSpec,
{
    /// Creates a registration from an owned provider factory.
    #[must_use]
    pub fn new<P>(descriptor: ProviderDescriptor, provider: P) -> Self
    where
        P: ServiceProvider<S>,
    {
        Self {
            descriptor,
            provider: Arc::new(provider),
        }
    }

    /// Creates a registration from an already shared provider factory.
    #[must_use]
    pub fn shared(descriptor: ProviderDescriptor, provider: Arc<dyn ServiceProvider<S>>) -> Self {
        Self {
            descriptor,
            provider,
        }
    }

    /// Gets registration metadata.
    #[must_use]
    pub fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }

    pub(crate) fn into_parts(self) -> (ProviderDescriptor, Arc<dyn ServiceProvider<S>>) {
        (self.descriptor, self.provider)
    }
}
