// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{
    ServiceSpec,
    SyncServiceSpec,
};

/// A synchronous service family used to check the split specification traits.
struct ByteCountSpec;

impl ServiceSpec for ByteCountSpec {
    type Config = [u8];
}

impl SyncServiceSpec for ByteCountSpec {
    type Output = usize;
}

/// Verifies that synchronous output is selected independently from config.
#[test]
fn test_sync_service_spec_selects_its_output_type() {
    fn assert_spec<S: SyncServiceSpec<Config = [u8], Output = usize>>() {}

    assert_spec::<ByteCountSpec>();
}
