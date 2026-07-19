// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous output capability for a service specification.

use crate::ServiceSpec;

/// Type-level output contract for asynchronously created services.
pub trait AsyncServiceSpec: ServiceSpec
where
    Self::Config: Sync,
{
    /// Complete output handle returned by asynchronous provider factories.
    type Output: Send + 'static;
}
