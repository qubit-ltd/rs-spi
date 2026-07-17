// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::ProviderSelectorError;
use qubit_spi::{
    ProviderId,
    ProviderSelector,
};

/// Verifies trimming and ASCII case normalization at the selector boundary.
#[test]
fn test_selector_normalizes_configuration_input() {
    assert_eq!(
        "git+ssh",
        ProviderSelector::parse(" Git+SSH ")
            .expect("valid selector should normalize")
            .as_str(),
    );
}

/// Verifies standard string parsing and string-reference conversion.
#[test]
fn test_selector_supports_standard_string_traits() {
    let selector = " Git+SSH "
        .parse::<ProviderSelector>()
        .expect("valid selector should parse");

    assert_eq!("git+ssh", AsRef::<str>::as_ref(&selector));
    assert_eq!("git+ssh", selector.to_string());
}

/// Verifies that a validated canonical provider ID converts without reparsing.
#[test]
fn test_provider_selector_from_provider_id() {
    let id = ProviderId::new("file-command")
        .expect("test provider ID should be valid");

    let selector = ProviderSelector::from(&id);

    assert_eq!("file-command", selector.as_str());
}

/// Verifies preservation of raw and normalized invalid selector input.
#[test]
fn test_selector_errors_preserve_raw_and_normalized_input() {
    let empty =
        ProviderSelector::parse("  ").expect_err("blank selector should fail");
    assert_eq!("  ", empty.input());
    let ProviderSelectorError::Empty { input, .. } = empty else {
        panic!("blank selector should retain the empty variant");
    };
    assert_eq!("  ", input.as_ref());

    let invalid = ProviderSelector::parse(" Bad Selector ")
        .expect_err("selector containing a space should fail");
    assert_eq!(" Bad Selector ", invalid.input());
    let ProviderSelectorError::Invalid {
        input, normalized, ..
    } = invalid
    else {
        panic!("invalid selector should retain both representations");
    };
    assert_eq!(" Bad Selector ", input.as_ref());
    assert_eq!("bad selector", normalized.as_ref());
}
