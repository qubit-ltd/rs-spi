// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Service model shared by the cross-crate provider fixture.

use std::convert::Infallible;

use spi::AsyncServiceSpec;
use spi::ServiceSpec;
use spi::SyncServiceSpec;

/// Service family shared by the cross-crate provider fixture.
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
