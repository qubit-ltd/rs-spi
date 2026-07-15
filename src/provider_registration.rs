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

/// One provider factory paired with the metadata needed to register it.
///
/// [`crate::ProviderRegistryBuilder`] registration methods create this container
/// internally and consume it while building the immutable registry. Callers
/// normally register a descriptor and factory through the builder instead of
/// constructing this intermediate value directly.
pub struct ProviderRegistration<S>
where
    S: ServiceSpec,
{
    /// Immutable ID, aliases, and priority used during registration.
    descriptor: ProviderDescriptor,
    /// Shared factory that creates the service for this registration.
    provider: Arc<dyn ServiceProvider<S>>,
}

impl<S> ProviderRegistration<S>
where
    S: ServiceSpec,
{
    /// Creates a registration from an owned provider factory.
    ///
    /// `descriptor` supplies provider metadata and `provider` is moved into a
    /// shared factory allocation. Returns the assembled registration.
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
    ///
    /// `descriptor` supplies provider metadata and `provider` is reused as the
    /// registration's factory. Returns the assembled registration.
    #[must_use]
    pub fn shared(descriptor: ProviderDescriptor, provider: Arc<dyn ServiceProvider<S>>) -> Self {
        Self {
            descriptor,
            provider,
        }
    }

    /// Returns the immutable registration metadata.
    #[must_use]
    pub fn descriptor(&self) -> &ProviderDescriptor {
        &self.descriptor
    }

    /// Splits this registration into the metadata and shared factory it owns.
    ///
    /// Returns the pair consumed by registry construction.
    pub(crate) fn into_parts(self) -> (ProviderDescriptor, Arc<dyn ServiceProvider<S>>) {
        (self.descriptor, self.provider)
    }
}
