// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderSelection;

#[test]
fn named_selection_normalizes_its_selector() {
    let selection = ProviderSelection::named(" File+Command ").unwrap();

    assert!(matches!(
        selection,
        ProviderSelection::Named(selector) if selector.as_str() == "file+command"
    ));
}

#[test]
fn chain_selection_preserves_candidate_order() {
    let selection = ProviderSelection::chain(["remote", "memory"]).unwrap();

    assert!(matches!(
        selection,
        ProviderSelection::Chain(selectors)
            if selectors.iter().map(|selector| selector.as_str()).collect::<Vec<_>>()
                == ["remote", "memory"]
    ));
}
