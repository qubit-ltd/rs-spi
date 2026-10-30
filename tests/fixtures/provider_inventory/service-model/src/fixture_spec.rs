// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Service family shared by the cross-crate provider fixture.

use std::convert::Infallible;

use spi::AsyncServiceSpec;
use spi::ServiceSpec;
use spi::SyncServiceSpec;

/// Service family shared by the cross-crate provider fixture.
///
/// # Examples
///
/// ```
/// use qubit_spi::{AsyncServiceSpec, ServiceSpec, SyncServiceSpec};
/// use service_model::FixtureSpec;
///
/// let _: <FixtureSpec as ServiceSpec>::Config = String::new();
/// let _: <FixtureSpec as ServiceSpec>::Error = std::convert::Infallible;
/// let _: <FixtureSpec as SyncServiceSpec>::Output = String::new();
/// let _: <FixtureSpec as AsyncServiceSpec>::Output = String::new();
/// ```
pub struct FixtureSpec;

impl ServiceSpec for FixtureSpec {
    type Config = String;
    type Error = Infallible;
}

impl SyncServiceSpec for FixtureSpec {
    type Output = String;
}

impl AsyncServiceSpec for FixtureSpec {
    type Output = String;
}
