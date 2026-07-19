// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::ProviderErrorKind;
use qubit_spi::{
    FallbackPolicy,
    ProviderSelection,
};

/// Verifies that every selection starts with absence-only fallback.
#[test]
fn test_selection_uses_on_absence_by_default() {
    assert_eq!(
        FallbackPolicy::OnAbsence,
        ProviderSelection::auto().fallback_policy()
    );
}

/// Verifies every fallback policy against every provider error kind.
#[test]
fn test_fallback_policy_allows_expected_error_kinds() {
    for (policy, kind, expected) in [
        (FallbackPolicy::Never, ProviderErrorKind::Unsupported, false),
        (FallbackPolicy::Never, ProviderErrorKind::Unavailable, false),
        (
            FallbackPolicy::Never,
            ProviderErrorKind::InvalidConfiguration,
            false,
        ),
        (
            FallbackPolicy::Never,
            ProviderErrorKind::InitializationFailed,
            false,
        ),
        (
            FallbackPolicy::OnAbsence,
            ProviderErrorKind::Unsupported,
            true,
        ),
        (
            FallbackPolicy::OnAbsence,
            ProviderErrorKind::Unavailable,
            true,
        ),
        (
            FallbackPolicy::OnAbsence,
            ProviderErrorKind::InvalidConfiguration,
            false,
        ),
        (
            FallbackPolicy::OnAbsence,
            ProviderErrorKind::InitializationFailed,
            false,
        ),
        (
            FallbackPolicy::OnAnyError,
            ProviderErrorKind::Unsupported,
            true,
        ),
        (
            FallbackPolicy::OnAnyError,
            ProviderErrorKind::Unavailable,
            true,
        ),
        (
            FallbackPolicy::OnAnyError,
            ProviderErrorKind::InvalidConfiguration,
            true,
        ),
        (
            FallbackPolicy::OnAnyError,
            ProviderErrorKind::InitializationFailed,
            true,
        ),
    ] {
        assert_eq!(expected, policy.allows(kind));
    }
}
