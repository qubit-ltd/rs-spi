// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{FallbackPolicy, ProviderSelection};

/// Verifies that every selection starts with absence-only fallback.
#[test]
fn test_selection_uses_on_absence_by_default() {
    assert_eq!(
        FallbackPolicy::OnAbsence,
        ProviderSelection::auto().fallback_policy()
    );
}
