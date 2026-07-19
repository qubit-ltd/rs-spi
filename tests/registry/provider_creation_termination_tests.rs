// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderCreationTermination;

/// Verifies that exhaustion and policy stops remain distinguishable.
#[test]
fn test_provider_creation_termination_values_are_distinct() {
    assert_ne!(
        ProviderCreationTermination::Exhausted,
        ProviderCreationTermination::StoppedByPolicy,
    );
}
