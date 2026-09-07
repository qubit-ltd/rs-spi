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
///
/// # Examples
///
/// ```rust
/// use qubit_spi::{ServiceSpec, SyncServiceSpec};
/// struct Spec;
/// impl ServiceSpec for Spec {
///     type Config = String;
///     type Error = std::io::Error;
/// }
/// impl SyncServiceSpec for Spec { type Output = String; }
/// let output: <Spec as SyncServiceSpec>::Output = "ready".to_owned();
/// assert_eq!("ready", output);
/// ```
pub trait SyncServiceSpec: ServiceSpec {
    /// Complete output handle returned by synchronous provider factories.
    type Output;
}
