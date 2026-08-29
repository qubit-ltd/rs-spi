// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Metadata-bearing asynchronous provider definitions.

use crate::AsyncServiceProvider;
use crate::AsyncServiceSpec;
use crate::ProviderMetadata;

/// Marker combining asynchronous creation with registration metadata.
///
/// # Type Parameters
///
/// * `S` - Asynchronous service family implemented by the provider.
pub trait AsyncProviderDefinition<S>: ProviderMetadata + AsyncServiceProvider<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    // empty
}

impl<S, T> AsyncProviderDefinition<S> for T
where
    S: AsyncServiceSpec,
    S::Config: Sync,
    T: ProviderMetadata + AsyncServiceProvider<S> + ?Sized,
{
    // empty
}
