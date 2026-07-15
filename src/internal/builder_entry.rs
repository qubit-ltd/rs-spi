// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private registrations retained until immutable registry construction.

use std::sync::Arc;

use crate::{ProviderDescriptor, ServiceProvider, ServiceSpec};

/// Descriptor and factory pair retained by a registry builder.
pub(crate) struct BuilderEntry<S>
where
    S: ServiceSpec,
{
    /// Immutable ID, aliases, and priority used during registration.
    pub(crate) descriptor: ProviderDescriptor,
    /// Shared factory that creates the service for this entry.
    pub(crate) provider: Arc<dyn ServiceProvider<S>>,
}
