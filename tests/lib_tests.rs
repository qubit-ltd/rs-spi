// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{FallbackPolicy, ProviderSelection};

/// Verifies the crate root re-exports core selection types.
#[test]
fn test_crate_root_reexports_core_selection_types() {
    let selection = ProviderSelection::auto().with_fallback_policy(FallbackPolicy::Never);
    assert_eq!(FallbackPolicy::Never, selection.fallback_policy());
}
