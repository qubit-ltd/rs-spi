// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderId;

/// Verifies the complete accepted canonical-token boundary.
#[test]
fn test_provider_id_accepts_canonical_token_boundaries() {
    for input in [
        "a",
        "0",
        "a0",
        "a_b",
        "a--b",
        "a..b",
        "git+ssh",
        "vendor.v2",
    ] {
        let id = ProviderId::new(input).expect("canonical test input should be valid");
        assert_eq!(input, id.as_str());
    }
}

/// Verifies standard string parsing and string-reference conversion.
#[test]
fn test_provider_id_supports_standard_string_traits() {
    let id = "vendor.v2"
        .parse::<ProviderId>()
        .expect("canonical provider ID should parse");

    assert_eq!("vendor.v2", AsRef::<str>::as_ref(&id));
}
