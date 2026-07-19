// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_spi::error::ProviderDescriptorError;
use qubit_spi::{
    ProviderDescriptor,
    ProviderId,
};

/// Verifies rejection of an alias equal to the canonical provider ID.
#[test]
fn test_descriptor_rejects_an_alias_that_duplicates_its_canonical_id() {
    let error =
        ProviderDescriptor::new(ProviderId::new("file-command").unwrap())
            .with_aliases(["file-command"])
            .unwrap_err();
    assert_eq!("file-command", error.alias());
    assert_eq!(
        "provider alias matches canonical ID: file-command",
        error.to_string()
    );
    assert!(Error::source(&error).is_none());
    assert!(matches!(
        error,
        ProviderDescriptorError::AliasMatchesId { .. }
    ));
}

/// Verifies invalid alias position, input, and source diagnostics.
#[test]
fn test_descriptor_reports_invalid_alias_with_position_and_source() {
    let error =
        ProviderDescriptor::new(ProviderId::new("file-command").unwrap())
            .with_aliases(["file", "bad alias"])
            .unwrap_err();
    assert_eq!("bad alias", error.alias());
    assert_eq!(
        "invalid provider alias at index 1: \"bad alias\"",
        error.to_string()
    );
    assert!(Error::source(&error).is_some());
    assert!(matches!(
        error,
        ProviderDescriptorError::InvalidAlias { alias_index: 1, .. }
    ));
}

/// Verifies invalid-alias diagnostics for a single-element alias input.
#[test]
fn test_descriptor_rejects_a_single_invalid_alias() {
    let error =
        ProviderDescriptor::new(ProviderId::new("file-command").unwrap())
            .with_aliases(["bad alias"])
            .unwrap_err();
    assert_eq!("bad alias", error.alias());
    assert!(matches!(
        error,
        ProviderDescriptorError::InvalidAlias { alias_index: 0, .. }
    ));
}

/// Verifies that normalized duplicate aliases have their own classification.
#[test]
fn test_descriptor_distinguishes_duplicate_aliases() {
    let error =
        ProviderDescriptor::new(ProviderId::new("file-command").unwrap())
            .with_aliases([" File ", "file"])
            .unwrap_err();
    assert_eq!("file", error.alias());
    assert_eq!("duplicate provider alias: file", error.to_string());
    assert!(matches!(
        error,
        ProviderDescriptorError::DuplicateAlias { .. }
    ));
}
