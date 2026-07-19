// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Registration metadata shared by synchronous and asynchronous providers.

use crate::ProviderDescriptor;

/// Contract for a provider with stable identity and selection metadata.
///
/// Registry registration snapshots the returned descriptor before acquiring
/// its write lock. Later provider state changes therefore cannot alter the
/// registered lookup metadata.
pub trait ProviderMetadata: Send + Sync + 'static {
    /// Returns this provider's registration metadata.
    ///
    /// # Returns
    ///
    /// A descriptor snapshot containing the canonical ID, aliases, and
    /// automatic-selection priority.
    #[must_use]
    fn descriptor(&self) -> ProviderDescriptor;
}
