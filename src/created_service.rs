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
    /// `provider_id` identifies the provider that produced `service`.
    /// Returns a value that retains both inputs without performing validation
    /// or service creation.
    #[must_use]
    pub fn new(provider_id: ProviderId, service: T) -> Self {
        Self {
            provider_id,
            service,
        }
    }

    /// Returns the canonical ID of the provider that created the service.
    ///
    /// The returned reference is valid for as long as this result is retained.
    #[must_use]
    #[inline]
    pub fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    /// Returns the created service handle or value.
    ///
    /// The returned reference is valid for as long as this result is retained.
    #[must_use]
    #[inline]
    pub fn service(&self) -> &T {
        &self.service
    }

    /// Consumes this result and returns the created service.
    ///
    /// Use this when provider identity is no longer needed and ownership of the
    /// service must be transferred to the caller.
    #[inline]
    pub fn into_service(self) -> T {
        self.service
    }
}
