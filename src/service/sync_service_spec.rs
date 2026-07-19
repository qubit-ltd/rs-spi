// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Synchronous output capability for a service specification.

use crate::ServiceSpec;

/// Type-level output contract for synchronously created services.
pub trait SyncServiceSpec: ServiceSpec {
    /// Complete output handle returned by synchronous provider factories.
    type Output;
}
