// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for resolution termination classifications.

use qubit_spi::ResolutionTermination;

#[test]
fn test_resolution_termination_values_are_distinct() {
    assert_ne!(
        ResolutionTermination::Exhausted,
        ResolutionTermination::StoppedByPolicy,
    );
}
