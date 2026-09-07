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
///
/// # Examples
///
/// ```rust
/// use qubit_spi::{ServiceSpec, AsyncServiceSpec};
/// struct Spec;
/// impl ServiceSpec for Spec { type Config = (); type Error = std::io::Error; }
/// impl AsyncServiceSpec for Spec { type Output = String; }
/// let future = async { String::from("ready") };
/// let output: <Spec as AsyncServiceSpec>::Output = futures::executor::block_on(future);
/// assert_eq!("ready", output);
/// ```
pub trait AsyncServiceSpec: ServiceSpec
where
    Self::Config: Sync,
{
    /// Complete output handle returned by asynchronous provider factories.
    type Output: Send + 'static;
}
