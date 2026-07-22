// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private immutable provider entries.

use std::sync::Arc;

use crate::ProviderDescriptor;

/// Internal pairing of one descriptor and the factory it represents.
pub(crate) struct RegistryEntry<P: ?Sized> {
    /// Metadata used to identify and order this provider.
    pub(crate) descriptor: Arc<ProviderDescriptor>,
    /// Shared factory used to create this provider's service.
    pub(crate) provider: Arc<P>,
}

impl<P: ?Sized> Clone for RegistryEntry<P> {
    /// Clones the shared descriptor and provider handles.
    ///
    /// # Returns
    ///
    /// An owned entry referring to the same provider definition.
    #[inline]
    fn clone(&self) -> Self {
        Self {
            descriptor: Arc::clone(&self.descriptor),
            provider: Arc::clone(&self.provider),
        }
    }
}
