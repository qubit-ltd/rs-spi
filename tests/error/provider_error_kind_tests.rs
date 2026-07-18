// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::ProviderErrorKind;

/// Verifies that provider error kinds remain distinct classifications.
#[test]
fn test_provider_error_kinds_are_distinct() {
    assert_ne!(
        ProviderErrorKind::Unsupported,
        ProviderErrorKind::Unavailable
    );
    assert_ne!(
        ProviderErrorKind::Unavailable,
        ProviderErrorKind::InvalidConfiguration,
    );
    assert_ne!(
        ProviderErrorKind::InvalidConfiguration,
        ProviderErrorKind::InitializationFailed,
    );
}
