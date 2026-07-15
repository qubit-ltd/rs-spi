// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Successfully created services with their winning provider identity.

use crate::ProviderId;

/// A service output paired with the canonical provider that created it.
///
/// Resolvers return this type when callers need both a usable service handle
/// and the identity used for observability, diagnostics, or later reporting.
#[derive(Debug)]
pub struct CreatedService<T> {
    /// Canonical identifier of the provider that successfully created `service`.
    provider_id: ProviderId,
    /// Successfully created service handle or value.
    service: T,
}

impl<T> CreatedService<T> {
    /// Creates a service result with its winning provider identity.
    ///
    /// # Arguments
    ///
    /// * `provider_id` - Canonical provider that produced `service`.
    /// * `service` - Successfully created output value.
    ///
    /// # Returns
    ///
    /// A result retaining both inputs without further service creation.
    #[inline]
    #[must_use]
    pub(crate) fn new(provider_id: ProviderId, service: T) -> Self {
        Self {
            provider_id,
            service,
        }
    }

    /// Returns the canonical ID of the provider that created the service.
    ///
    /// # Returns
    ///
    /// The winning provider's canonical ID.
    #[inline(always)]
    #[must_use]
    pub fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    /// Returns the created service handle or value.
    ///
    /// # Returns
    ///
    /// A shared reference to the created service output.
    #[inline(always)]
    #[must_use]
    pub fn service(&self) -> &T {
        &self.service
    }

    /// Consumes this result and returns the created service.
    ///
    /// # Returns
    ///
    /// The owned service output without its provider identity.
    #[inline(always)]
    pub fn into_service(self) -> T {
        self.service
    }

    /// Consumes this result and returns its provider identity and service.
    ///
    /// # Returns
    ///
    /// Both owned fields in provider-ID then service order.
    #[inline(always)]
    pub fn into_parts(self) -> (ProviderId, T) {
        (self.provider_id, self.service)
    }
}
