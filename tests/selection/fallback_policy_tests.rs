// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::FallbackPolicy;
use qubit_spi::ProviderSelection;
use qubit_spi::error::ProviderFailureKind;

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
fn test_fallback_policy_continues_after_expected_failure_kinds() {
    for (policy, kind, expected) in [
        (
            FallbackPolicy::Never,
            ProviderFailureKind::Unsupported,
            false,
        ),
        (
            FallbackPolicy::Never,
            ProviderFailureKind::Unavailable,
            false,
        ),
        (
            FallbackPolicy::Never,
            ProviderFailureKind::InvalidConfiguration,
            false,
        ),
        (
            FallbackPolicy::Never,
            ProviderFailureKind::InitializationFailed,
            false,
        ),
        (
            FallbackPolicy::OnAbsence,
            ProviderFailureKind::Unsupported,
            true,
        ),
        (
            FallbackPolicy::OnAbsence,
            ProviderFailureKind::Unavailable,
            true,
        ),
        (
            FallbackPolicy::OnAbsence,
            ProviderFailureKind::InvalidConfiguration,
            false,
        ),
        (
            FallbackPolicy::OnAbsence,
            ProviderFailureKind::InitializationFailed,
            false,
        ),
        (
            FallbackPolicy::OnAnyError,
            ProviderFailureKind::Unsupported,
            true,
        ),
        (
            FallbackPolicy::OnAnyError,
            ProviderFailureKind::Unavailable,
            true,
        ),
        (
            FallbackPolicy::OnAnyError,
            ProviderFailureKind::InvalidConfiguration,
            true,
        ),
        (
            FallbackPolicy::OnAnyError,
            ProviderFailureKind::InitializationFailed,
            true,
        ),
    ] {
        assert_eq!(expected, policy.should_continue_after(kind));
    }
}
