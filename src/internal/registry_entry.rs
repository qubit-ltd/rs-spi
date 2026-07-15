// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private immutable provider entries.

use std::sync::Arc;

use crate::{
    ProviderDescriptor,
    ServiceProvider,
    ServiceSpec,
};

/// Internal pairing of one descriptor and the factory it represents.
pub(crate) struct RegistryEntry<S>
where
    S: ServiceSpec,
{
    /// Metadata used to identify and order this provider.
    pub(crate) descriptor: ProviderDescriptor,
    /// Shared factory used to create this provider's service.
    pub(crate) provider: Arc<dyn ServiceProvider<S>>,
}
