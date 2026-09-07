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
///
/// # Examples
///
/// ```rust
/// use qubit_spi::{ProviderDescriptor, ProviderMetadata, provider_descriptor};
/// struct Local;
/// impl ProviderMetadata for Local {
///     fn descriptor(&self) -> ProviderDescriptor {
///         provider_descriptor!("local", aliases: ["disk"], priority: 10)
///     }
/// }
/// assert_eq!("disk", Local.descriptor().aliases()[0].as_str());
/// ```
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
