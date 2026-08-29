// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderId;
use qubit_spi::ProviderSelector;

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
    let id = ProviderId::new("file-command").expect("test provider ID should be valid");

    let selector = ProviderSelector::from(&id);

    assert_eq!("file-command", selector.as_str());
}
