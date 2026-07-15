// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_spi::{
    ProviderSelection, ProviderSelectionErrorKind, ProviderSelectionKind, ProviderSelector,
};

#[test]
fn named_selection_normalizes_its_selector() {
    let selection = ProviderSelection::named(" File+Command ").unwrap();

    assert_eq!(ProviderSelectionKind::Named, selection.kind());
    assert_eq!(
        Some("file+command"),
        selection.selector().map(ProviderSelector::as_str),
    );
    assert!(selection.selectors().is_empty());
}

#[test]
fn chain_selection_preserves_candidate_order() {
    let selection = ProviderSelection::chain(["remote", "memory"]).unwrap();

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

#[test]
fn selection_construction_enforces_invariants() {
    let automatic = ProviderSelection::auto();
    assert_eq!(ProviderSelectionKind::Auto, automatic.kind());
    assert_eq!(automatic, ProviderSelection::default());

    let empty = ProviderSelection::chain(Vec::<&str>::new()).expect_err("empty chain should fail");
    assert_eq!(ProviderSelectionErrorKind::EmptyChain, empty.kind());

    let invalid = ProviderSelection::chain(["valid", "bad selector"])
        .expect_err("invalid chain selector should fail");
    assert_eq!(ProviderSelectionErrorKind::InvalidSelector, invalid.kind());
    assert_eq!(Some(1), invalid.selector_index());
    assert_eq!(Some("bad selector"), invalid.selector_input());
    assert!(Error::source(&invalid).is_some());
}
