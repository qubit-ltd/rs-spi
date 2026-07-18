// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Self-described providers accepted by a provider registry.

use crate::{
    ProviderDescriptor,
    ServiceProvider,
    ServiceSpec,
};

/// Registration contract for a provider with stable identity and metadata.
///
/// Implement this trait for providers that may be inserted into a
/// [`crate::ProviderRegistry`]. Registry registration snapshots the returned
/// descriptor, so later provider state changes cannot alter registered lookup
/// metadata.
pub trait ProviderDefinition<S>: ServiceProvider<S>
where
    S: ServiceSpec,
{
    /// Returns this provider's registration metadata.
    ///
    /// # Returns
    ///
    /// A descriptor snapshot containing the canonical ID, aliases, and
    /// automatic-selection priority.
    #[must_use]
    fn descriptor(&self) -> ProviderDescriptor;
}
