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
    ProviderSelection,
    ProviderSelectionTargetRef,
};

/// Verifies automatic defaults and invalid chain construction boundaries.
#[test]
fn test_selection_construction_enforces_invariants() {
    let automatic = ProviderSelection::auto();
    assert!(matches!(
        automatic.target(),
        ProviderSelectionTargetRef::Auto,
    ));
    assert_eq!(automatic, ProviderSelection::default());
    let empty = ProviderSelection::chain(Vec::<&str>::new()).unwrap_err();
    assert!(Error::source(&empty).is_none());
    assert_eq!(
        "provider selection chain must not be empty",
        empty.to_string(),
    );
    assert!(empty.selector_error().is_none());
    assert_eq!(None, empty.selector_index());
    assert!(empty.is_empty_chain());
    assert!(matches!(empty, ProviderSelectionBuildError::EmptyChain));
    let invalid =
        ProviderSelection::chain(["valid", "bad selector"]).unwrap_err();
    assert!(Error::source(&invalid).is_some());
    assert_eq!("bad selector", invalid.selector_error().unwrap().input());
    assert_eq!(Some(1), invalid.selector_index());
    assert!(!invalid.is_empty_chain());
    assert!(matches!(
        invalid,
        ProviderSelectionBuildError::InvalidSelector {
            selector_index: Some(1),
            ..
        }
    ));
    assert_eq!(
        "invalid provider selector at selection index 1: \"bad selector\"",
        invalid.to_string(),
    );
}

/// Verifies invalid named selections omit a position and retain their source.
#[test]
fn test_invalid_named_selection_preserves_input_and_source() {
    let error = ProviderSelection::named("bad selector").unwrap_err();
    assert!(Error::source(&error).is_some());
    assert_eq!("bad selector", error.selector_error().unwrap().input());
    assert_eq!(None, error.selector_index());
    assert!(!error.is_empty_chain());
    assert_eq!(
        "invalid provider selector \"bad selector\"",
        error.to_string(),
    );
    assert!(matches!(
        error,
        ProviderSelectionBuildError::InvalidSelector {
            selector_index: None,
            ..
        }
    ));
}
