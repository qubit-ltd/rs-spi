// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for stable provider-selector-error classifications.

use qubit_spi::error::ProviderSelectorErrorKind;

#[test]
fn test_provider_selector_error_kind_values_are_distinct() {
    assert_ne!(
        ProviderSelectorErrorKind::Empty,
        ProviderSelectorErrorKind::Invalid,
    );
}
