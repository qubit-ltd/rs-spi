// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous provider factories submitted through link-time discovery.

use std::sync::Arc;

use crate::AsyncProviderDefinition;
use crate::AsyncServiceSpec;
use crate::inventory::ProviderRegistrationSource;

/// An asynchronous provider factory and its declaration source.
///
/// This type supports macro expansion only; application code should use a
/// service family's generated inventory module instead.
pub struct AsyncProviderInventoryEntry<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    factory: fn() -> Arc<dyn AsyncProviderDefinition<S>>,
    source: ProviderRegistrationSource,
}

impl<S> AsyncProviderInventoryEntry<S>
where
    S: AsyncServiceSpec,
    S::Config: Sync,
{
    /// Creates a discovered asynchronous provider entry.
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
    pub const fn new(factory: fn() -> Arc<dyn AsyncProviderDefinition<S>>, source: ProviderRegistrationSource) -> Self {
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

    /// Invokes the submitted asynchronous provider factory.
    ///
    /// # Returns
    ///
    /// A shared asynchronous provider definition.
    ///
    /// # Panics
    ///
    /// Propagates any panic raised by the submitted factory.
    #[doc(hidden)]
    #[must_use]
    pub fn create_provider(&self) -> Arc<dyn AsyncProviderDefinition<S>> {
        (self.factory)()
    }
}
