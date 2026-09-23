// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared public service-family type for cross-crate inventory tests.

use std::convert::Infallible;

use qubit_spi::AsyncServiceSpec;
use qubit_spi::ServiceSpec;
use qubit_spi::SyncServiceSpec;

/// Service family used by external inventory macro expansion tests.
pub struct ExternalStringSpec;

impl ServiceSpec for ExternalStringSpec {
    type Config = String;
    type Error = Infallible;
}

impl SyncServiceSpec for ExternalStringSpec {
    type Output = String;
}

impl AsyncServiceSpec for ExternalStringSpec {
    type Output = String;
}
