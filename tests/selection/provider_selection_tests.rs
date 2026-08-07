// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::FallbackPolicy;
use qubit_spi::MissingProviderPolicy;
use qubit_spi::ProviderSelection;
use qubit_spi::ProviderSelectionTargetRef;

/// Verifies normalization and accessors for a named selection.
#[test]
fn test_named_selection_normalizes_its_selector() {
    let selection = ProviderSelection::named(" File+Command ")
        .expect("valid named selector should parse");

    assert!(matches!(
        selection.target(),
        ProviderSelectionTargetRef::Named(selector)
            if selector.as_str() == "file+command"
    ));
}

/// Verifies that chained selection preserves caller-supplied ordering.
#[test]
fn test_chain_selection_preserves_candidate_order() {
    let selection = ProviderSelection::chain(["remote", "memory"])
        .expect("valid selector chain should parse");

    assert!(matches!(
        selection.target(),
        ProviderSelectionTargetRef::Chain {
            selectors,
            missing_policy: MissingProviderPolicy::Reject,
        } if selectors
            .iter()
            .map(|selector| selector.as_str())
            .eq(["remote", "memory"])
    ));
}

/// Verifies that the public target view preserves every selection shape.
#[test]
fn test_selection_target_preserves_named_and_chain_shape() {
    let named = ProviderSelection::named(" File ")
        .expect("named selector should parse");
    let chain = ProviderSelection::chain(["file"])
        .expect("chain selector should parse");

    assert!(matches!(
        named.target(),
        ProviderSelectionTargetRef::Named(selector)
            if selector.as_str() == "file"
    ));
    assert!(matches!(
        chain.target(),
        ProviderSelectionTargetRef::Chain {
            selectors,
            missing_policy: MissingProviderPolicy::Reject,
        } if selectors.len() == 1 && selectors[0].as_str() == "file"
    ));
}

/// Verifies that lenient chain construction is explicit and observable.
#[test]
fn test_chain_allowing_missing_exposes_ignore_policy() {
    let selection =
        ProviderSelection::chain_allowing_missing(["optional", "fallback"])
            .expect("optional chain should parse");

    assert!(matches!(
        selection.target(),
        ProviderSelectionTargetRef::Chain {
            missing_policy: MissingProviderPolicy::Ignore,
            ..
        }
    ));
}

/// Verifies that automatic selection has a distinct public target shape.
#[test]
fn test_auto_selection_exposes_auto_target() {
    assert!(matches!(
        ProviderSelection::auto().target(),
        ProviderSelectionTargetRef::Auto,
    ));
}

/// Verifies immutable fallback-policy replacement without changing the target.
#[test]
fn test_selection_replaces_its_fallback_policy_immutably() {
    let original = ProviderSelection::named("memory")
        .expect("test selector should be valid");
    let replaced = original.clone().with_fallback_policy(FallbackPolicy::Never);

    assert_eq!(FallbackPolicy::OnAbsence, original.fallback_policy());
    assert_eq!(FallbackPolicy::Never, replaced.fallback_policy());
    assert_eq!(original.target(), replaced.target());
}
