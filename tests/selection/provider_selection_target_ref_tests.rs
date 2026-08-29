// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderSelection;
use qubit_spi::ProviderSelectionTargetRef;

/// Verifies that target views are copyable borrowed snapshots.
#[test]
fn test_provider_selection_target_ref_is_copyable_and_debuggable() {
    let selection = ProviderSelection::named("memory").expect("test selector should parse");
    let target = selection.target();
    let copied = target;

    assert_eq!(target, copied);
    assert!(matches!(copied, ProviderSelectionTargetRef::Named(_)));
    assert!(format!("{copied:?}").contains("memory"));
}
