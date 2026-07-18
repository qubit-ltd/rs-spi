// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_spi::error::ProviderSelectionBuildError;
use qubit_spi::{
    FallbackPolicy,
    ProviderSelection,
    ProviderSelector,
};

/// Verifies normalization and accessors for a named selection.
#[test]
fn test_named_selection_normalizes_its_selector() {
    let selection = ProviderSelection::named(" File+Command ")
        .expect("valid named selector should parse");

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
    let selection = ProviderSelection::chain(["remote", "memory"])
        .expect("valid selector chain should parse");

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

/// Verifies that every selection starts with absence-only fallback.
#[test]
fn test_selection_uses_on_absence_by_default() {
    assert_eq!(
        FallbackPolicy::OnAbsence,
        ProviderSelection::auto().fallback_policy(),
    );
}

/// Verifies immutable fallback-policy replacement without changing the target.
#[test]
fn test_selection_replaces_its_fallback_policy_immutably() {
    let original = ProviderSelection::named("memory")
        .expect("test selector should be valid");
    let replaced = original.clone().with_fallback_policy(FallbackPolicy::Never);

    assert_eq!(FallbackPolicy::OnAbsence, original.fallback_policy());
    assert_eq!(FallbackPolicy::Never, replaced.fallback_policy());
    assert_eq!(original.selectors(), replaced.selectors());
}

/// Verifies automatic defaults and invalid chain construction boundaries.
#[test]
fn test_selection_construction_enforces_invariants() {
    let automatic = ProviderSelection::auto();
    assert!(automatic.selectors().is_empty());
    assert_eq!(automatic, ProviderSelection::default());

    let empty = ProviderSelection::chain(Vec::<&str>::new())
        .expect_err("empty chain should fail");
    assert!(Error::source(&empty).is_none());
    assert_eq!(
        "provider selection chain must not be empty",
        empty.to_string(),
    );
    assert!(matches!(empty, ProviderSelectionBuildError::EmptyChain));

    let invalid = ProviderSelection::chain(["valid", "bad selector"])
        .expect_err("invalid chain selector should fail");
    assert!(Error::source(&invalid).is_some());
    assert_eq!(
        "invalid provider selector at selection index 1: \"bad selector\"",
        invalid.to_string(),
    );
    let ProviderSelectionBuildError::InvalidSelector {
        selector_index,
        source,
        ..
    } = invalid
    else {
        panic!("invalid chain selector should retain its context");
    };
    assert_eq!(Some(1), selector_index);
    assert_eq!("bad selector", source.input());
}

/// Verifies invalid named selections omit a position and retain their source.
#[test]
fn test_invalid_named_selection_preserves_input_and_source() {
    let error = ProviderSelection::named("bad selector")
        .expect_err("invalid named selector should fail");

    assert!(Error::source(&error).is_some());
    assert_eq!(
        "invalid provider selector \"bad selector\"",
        error.to_string(),
    );
    let ProviderSelectionBuildError::InvalidSelector {
        selector_index,
        source,
        ..
    } = error
    else {
        panic!("invalid named selector should retain its context");
    };
    assert_eq!(None, selector_index);
    assert_eq!("bad selector", source.input());
}
