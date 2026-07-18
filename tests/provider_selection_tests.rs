// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{FallbackPolicy, ProviderSelection, ProviderSelector};

/// Verifies normalization and accessors for a named selection.
#[test]
fn test_named_selection_normalizes_its_selector() {
    let selection =
        ProviderSelection::named(" File+Command ").expect("valid named selector should parse");

    assert_eq!(
        ["file+command"],
        selection
            .selectors()
            .iter()
            .map(ProviderSelector::as_str)
            .collect::<Vec<_>>()
            .as_slice(),
    );
}

/// Verifies that chained selection preserves caller-supplied ordering.
#[test]
fn test_chain_selection_preserves_candidate_order() {
    let selection =
        ProviderSelection::chain(["remote", "memory"]).expect("valid selector chain should parse");

    assert_eq!(
        ["remote", "memory"],
        selection
            .selectors()
            .iter()
            .map(ProviderSelector::as_str)
            .collect::<Vec<_>>()
            .as_slice(),
    );
}

/// Verifies immutable fallback-policy replacement without changing the target.
#[test]
fn test_selection_replaces_its_fallback_policy_immutably() {
    let original = ProviderSelection::named("memory").expect("test selector should be valid");
    let replaced = original.clone().with_fallback_policy(FallbackPolicy::Never);

    assert_eq!(FallbackPolicy::OnAbsence, original.fallback_policy());
    assert_eq!(FallbackPolicy::Never, replaced.fallback_policy());
    assert_eq!(original.selectors(), replaced.selectors());
}
