// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
// =============================================================================
//! Synchronous provider factories submitted through link-time discovery.

use std::sync::Arc;

use crate::ProviderDefinition;
use crate::SyncServiceSpec;
use crate::inventory::ProviderRegistrationSource;

/// A synchronous provider factory and its declaration source.
///
/// This type supports macro expansion only; application code should use a
/// service family's generated inventory module instead.
pub struct SyncProviderInventoryEntry<S>
where
    S: SyncServiceSpec,
{
    factory: fn() -> Arc<dyn ProviderDefinition<S>>,
    source: ProviderRegistrationSource,
}

impl<S> SyncProviderInventoryEntry<S>
where
    S: SyncServiceSpec,
{
    /// Creates a discovered synchronous provider entry.
    ///
    /// # Parameters
    ///
    /// * `factory` - Function that constructs one shared provider definition.
    /// * `source` - Static location at which the factory was submitted.
    ///
    /// # Returns
    ///
    /// An entry retaining the factory and its declaration source.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(factory: fn() -> Arc<dyn ProviderDefinition<S>>, source: ProviderRegistrationSource) -> Self {
        Self { factory, source }
    }

    /// Returns the static source location of this submission.
    ///
    /// # Returns
    ///
    /// The declaration location used to order discovery entries.
    #[doc(hidden)]
    #[must_use]
    pub const fn source(&self) -> ProviderRegistrationSource {
        self.source
    }

    /// Invokes the submitted provider factory.
    ///
    /// # Returns
    ///
    /// A shared synchronous provider definition.
    ///
    /// # Panics
    ///
    /// Propagates any panic raised by the submitted factory.
    #[doc(hidden)]
    #[must_use]
    pub fn create_provider(&self) -> Arc<dyn ProviderDefinition<S>> {
        (self.factory)()
    }
}
