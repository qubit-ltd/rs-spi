// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{
    AsyncServiceSpec,
    ServiceSpec,
    SyncServiceSpec,
};

/// Service family pairing string configuration with string output.
pub(crate) struct StringSpec;

impl ServiceSpec for StringSpec {
    type Config = String;
}

impl SyncServiceSpec for StringSpec {
    type Output = String;
}

impl AsyncServiceSpec for StringSpec {
    type Output = String;
}
