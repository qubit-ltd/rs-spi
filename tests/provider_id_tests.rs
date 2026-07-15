// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{ProviderId, ProviderIdErrorKind};

#[test]
fn provider_id_requires_canonical_values_but_accepts_uri_style_values() {
    assert!(ProviderId::new("git+ssh").is_ok());
    assert!(ProviderId::new("vendor.v2").is_ok());
    assert!(ProviderId::new("File").is_err());
    assert!(ProviderId::new(" file").is_err());
}

#[test]
fn provider_id_reports_empty_and_noncanonical_input() {
    let empty = ProviderId::new("").expect_err("empty ID should fail");
    assert_eq!(ProviderIdErrorKind::Empty, empty.kind());
    assert_eq!(Some(""), empty.input());

    for input in ["File", " file", "file-", "-file", "文件"] {
        let error = ProviderId::new(input).expect_err("noncanonical ID should fail");
        assert_eq!(ProviderIdErrorKind::NonCanonical, error.kind());
        assert_eq!(Some(input), error.input());
    }
}
