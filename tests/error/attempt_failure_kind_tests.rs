// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for stable attempt-failure classifications.

use qubit_spi::error::AttemptFailureKind;

#[test]
fn test_attempt_failure_kind_values_are_distinct() {
    assert_ne!(
        AttemptFailureKind::UnknownProvider,
        AttemptFailureKind::ProviderError,
    );
}
