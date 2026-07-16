// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_spi::error::ProviderSelectionError;
use qubit_spi::{
    ProviderSelection,
    ProviderSelectionKind,
    ProviderSelector,
};

/// Verifies normalization and accessors for a named selection.
#[test]
fn test_named_selection_normalizes_its_selector() {
    let selection = ProviderSelection::named(" File+Command ")
        .expect("valid named selector should parse");

    assert_eq!(ProviderSelectionKind::Named, selection.kind());
    assert_eq!(
        Some("file+command"),
        selection.selector().map(ProviderSelector::as_str),
    );
    assert!(selection.selectors().is_empty());
}

/// Verifies that chained selection preserves caller-supplied ordering.
#[test]
fn test_chain_selection_preserves_candidate_order() {
    let selection = ProviderSelection::chain(["remote", "memory"])
        .expect("valid selector chain should parse");

    assert_eq!(ProviderSelectionKind::Chain, selection.kind());
    assert_eq!(None, selection.selector());
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

/// Verifies automatic defaults and invalid chain construction boundaries.
#[test]
fn test_selection_construction_enforces_invariants() {
    let automatic = ProviderSelection::auto();
    assert_eq!(ProviderSelectionKind::Auto, automatic.kind());
    assert_eq!(automatic, ProviderSelection::default());

    let empty = ProviderSelection::chain(Vec::<&str>::new())
        .expect_err("empty chain should fail");
    assert!(Error::source(&empty).is_none());
    assert!(matches!(empty, ProviderSelectionError::EmptyChain));

    let invalid = ProviderSelection::chain(["valid", "bad selector"])
        .expect_err("invalid chain selector should fail");
    assert!(Error::source(&invalid).is_some());
    let ProviderSelectionError::InvalidSelector {
        selector_index,
        selector_input,
        source: _,
    } = invalid
    else {
        panic!("invalid chain selector should retain its context");
    };
    assert_eq!(1, selector_index);
    assert_eq!("bad selector", selector_input.as_ref());
}

/// Verifies invalid named selections report position zero and retain a source.
#[test]
fn test_invalid_named_selection_preserves_input_and_source() {
    let error = ProviderSelection::named("bad selector")
        .expect_err("invalid named selector should fail");

    assert!(Error::source(&error).is_some());
    let ProviderSelectionError::InvalidSelector {
        selector_index,
        selector_input,
        source: _,
    } = error
    else {
        panic!("invalid named selector should retain its context");
    };
    assert_eq!(0, selector_index);
    assert_eq!("bad selector", selector_input.as_ref());
}
