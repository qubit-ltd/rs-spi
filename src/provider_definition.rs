// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Self-described providers accepted by a provider registry.

use crate::{
    ProviderMetadata,
    ServiceProvider,
    SyncServiceSpec,
};

/// Marker combining synchronous creation with registration metadata.
///
/// Every type implementing both [`ProviderMetadata`] and
/// [`ServiceProvider<S>`] automatically implements this trait.
pub trait ProviderDefinition<S>: ProviderMetadata + ServiceProvider<S>
where
    S: SyncServiceSpec,
{
}

impl<S, T> ProviderDefinition<S> for T
where
    S: SyncServiceSpec,
    T: ProviderMetadata + ServiceProvider<S> + ?Sized,
{
}
