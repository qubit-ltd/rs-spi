// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{
    ProviderId,
    ProviderIdErrorKind,
};

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
        let id = ProviderId::new(input)
            .expect("canonical test input should be valid");
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

/// Verifies empty and every representative noncanonical input class.
#[test]
fn test_provider_id_reports_empty_and_noncanonical_input() {
    let empty = ProviderId::new("").expect_err("empty ID should fail");
    assert_eq!(ProviderIdErrorKind::Empty, empty.kind());
    assert_eq!(Some(""), empty.input());

    for input in [
        "File",
        " file",
        "file ",
        "file-",
        "-file",
        "file/name",
        "file:name",
        "file name",
        "file\nname",
        "文件",
    ] {
        let error =
            ProviderId::new(input).expect_err("noncanonical ID should fail");
        assert_eq!(ProviderIdErrorKind::NonCanonical, error.kind());
        assert_eq!(Some(input), error.input());
    }
}
