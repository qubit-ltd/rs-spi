// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{ProviderError, ProviderErrorKind};

#[test]
fn provider_error_preserves_kind_reason_and_source() {
    let error = ProviderError::unavailable_with_source(
        "file executable is absent",
        std::io::Error::other("ENOENT"),
    );

    assert_eq!(ProviderErrorKind::Unavailable, error.kind());
    assert_eq!("file executable is absent", error.reason());
    assert!(std::error::Error::source(&error).is_some());
}
