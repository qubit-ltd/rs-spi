// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::ProviderFailureKind;

/// Verifies failure kinds are distinct and identify absence precisely.
#[test]
fn test_provider_failure_kinds_are_distinct_and_classify_absence() {
    assert_ne!(
        ProviderFailureKind::Unsupported,
        ProviderFailureKind::Unavailable,
    );
    assert_ne!(
        ProviderFailureKind::InvalidConfiguration,
        ProviderFailureKind::InitializationFailed,
    );
    assert!(ProviderFailureKind::Unsupported.is_absence());
    assert!(ProviderFailureKind::Unavailable.is_absence());
    assert!(!ProviderFailureKind::InvalidConfiguration.is_absence());
    assert!(!ProviderFailureKind::InitializationFailed.is_absence());
}
