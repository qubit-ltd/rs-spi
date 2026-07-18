// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_spi::error::ProviderSelectionBuildError;
use qubit_spi::ProviderSelection;

/// Verifies automatic defaults and invalid chain construction boundaries.
#[test]
fn test_selection_construction_enforces_invariants() {
    let automatic = ProviderSelection::auto();
    assert!(automatic.selectors().is_empty());
    assert_eq!(automatic, ProviderSelection::default());
    let empty = ProviderSelection::chain(Vec::<&str>::new()).unwrap_err();
    assert!(Error::source(&empty).is_none());
    assert!(matches!(empty, ProviderSelectionBuildError::EmptyChain));
    let invalid = ProviderSelection::chain(["valid", "bad selector"]).unwrap_err();
    assert!(Error::source(&invalid).is_some());
    assert!(matches!(
        invalid,
        ProviderSelectionBuildError::InvalidSelector {
            selector_index: Some(1),
            ..
        }
    ));
}

/// Verifies invalid named selections omit a position and retain their source.
#[test]
fn test_invalid_named_selection_preserves_input_and_source() {
    let error = ProviderSelection::named("bad selector").unwrap_err();
    assert!(Error::source(&error).is_some());
    assert!(matches!(
        error,
        ProviderSelectionBuildError::InvalidSelector {
            selector_index: None,
            ..
        }
    ));
}
