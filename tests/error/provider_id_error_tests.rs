// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderId;
use qubit_spi::error::ProviderIdError;

/// Verifies empty and representative noncanonical input classifications.
#[test]
fn test_provider_id_reports_input_classes() {
    let empty = ProviderId::new("").unwrap_err();
    assert_eq!("", empty.input());
    assert!(matches!(empty, ProviderIdError::Empty { .. }));
    for input in [
        "File",
        " file",
        "file ",
        "file-",
        "-file",
        "file/name",
        "文件",
    ] {
        assert!(matches!(
            ProviderId::new(input),
            Err(ProviderIdError::NonCanonical { .. })
        ));
    }
}

/// Verifies that noncanonical IDs preserve escaped input in diagnostics.
#[test]
fn test_provider_id_display_quotes_noncanonical_input() {
    let error = ProviderId::new("file\nname").unwrap_err();
    assert_eq!("file\nname", error.input());
    assert_eq!(
        "provider ID is not canonical: \"file\\nname\"",
        error.to_string()
    );
}
