// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for stable provider-selection-error classifications.

use qubit_spi::error::ProviderSelectionErrorKind;

#[test]
fn test_provider_selection_error_kind_values_are_distinct() {
    assert_ne!(
        ProviderSelectionErrorKind::InvalidSelector,
        ProviderSelectionErrorKind::EmptyChain,
    );
}
