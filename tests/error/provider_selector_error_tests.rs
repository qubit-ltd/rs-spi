// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderSelector;
use qubit_spi::error::ProviderSelectorError;

/// Verifies preservation of raw and normalized invalid selector input.
#[test]
fn test_selector_errors_preserve_raw_and_normalized_input() {
    let empty = ProviderSelector::parse("  ").unwrap_err();
    assert_eq!("  ", empty.input());
    assert!(matches!(empty, ProviderSelectorError::Empty { .. }));
    let invalid = ProviderSelector::parse(" Bad Selector ").unwrap_err();
    let ProviderSelectorError::Invalid {
        input, normalized, ..
    } = invalid
    else {
        panic!("invalid selector should retain both representations");
    };
    assert_eq!(" Bad Selector ", input.as_ref());
    assert_eq!("bad selector", normalized.as_ref());
}
