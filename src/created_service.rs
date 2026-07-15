// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Successfully created services with their winning provider identity.

use crate::ProviderId;

/// A service output paired with the provider that created it.
#[derive(Debug)]
pub struct CreatedService<T> {
    provider_id: ProviderId,
    service: T,
}

impl<T> CreatedService<T> {
    /// Creates a service result.
    #[must_use]
    pub fn new(provider_id: ProviderId, service: T) -> Self {
        Self {
            provider_id,
            service,
        }
    }

    /// Gets the canonical ID of the provider that created the service.
    #[must_use]
    pub fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    /// Gets the created service.
    #[must_use]
    pub fn service(&self) -> &T {
        &self.service
    }

    /// Consumes this value and returns the service.
    pub fn into_service(self) -> T {
        self.service
    }
}
