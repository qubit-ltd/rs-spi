// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_spi::{
    ProviderDescriptor,
    ProviderDescriptorErrorKind,
    ProviderId,
};

/// Verifies that a descriptor retains typed ID, alias, and priority metadata.
#[test]
fn test_descriptor_keeps_typed_metadata() {
    let descriptor = ProviderDescriptor::new(
        ProviderId::new("file-command")
            .expect("test provider ID should be valid"),
    )
    .with_aliases(["file", "command"])
    .expect("test aliases should be valid")
    .with_priority(20);

    assert_eq!("file-command", descriptor.id().as_str());
    assert_eq!(20, descriptor.priority());
    assert_eq!(
        vec!["file", "command"],
        descriptor
            .aliases()
            .iter()
            .map(|alias| alias.as_str())
            .collect::<Vec<_>>(),
    );
}

/// Verifies rejection of an alias equal to the canonical provider ID.
#[test]
fn test_descriptor_rejects_an_alias_that_duplicates_its_canonical_id() {
    let error = ProviderDescriptor::new(
        ProviderId::new("file-command")
            .expect("test provider ID should be valid"),
    )
    .with_aliases(["file-command"])
    .expect_err("an alias matching the canonical ID should fail");

    assert_eq!(ProviderDescriptorErrorKind::AliasMatchesId, error.kind());
    assert_eq!(Some("file-command"), error.alias());
}

/// Verifies invalid alias position, input, and source diagnostics.
#[test]
fn test_descriptor_reports_invalid_alias_with_position_and_source() {
    let error = ProviderDescriptor::new(
        ProviderId::new("file-command").expect("valid ID"),
    )
    .with_aliases(["file", "bad alias"])
    .expect_err("invalid alias should fail");

    assert_eq!(ProviderDescriptorErrorKind::InvalidAlias, error.kind());
    assert_eq!(Some(1), error.alias_index());
    assert_eq!(Some("bad alias"), error.alias());
    assert!(Error::source(&error).is_some());
}

/// Verifies invalid-alias diagnostics for a single-element alias input.
#[test]
fn test_descriptor_rejects_a_single_invalid_alias() {
    let error = ProviderDescriptor::new(
        ProviderId::new("file-command").expect("valid ID"),
    )
    .with_aliases(["bad alias"])
    .expect_err("invalid alias should fail");

    assert_eq!(ProviderDescriptorErrorKind::InvalidAlias, error.kind());
    assert_eq!(Some(0), error.alias_index());
    assert_eq!(Some("bad alias"), error.alias());
}

/// Verifies that normalized duplicate aliases have their own classification.
#[test]
fn test_descriptor_distinguishes_duplicate_aliases() {
    let error = ProviderDescriptor::new(
        ProviderId::new("file-command").expect("valid ID"),
    )
    .with_aliases([" File ", "file"])
    .expect_err("normalized duplicate aliases should fail");

    assert_eq!(ProviderDescriptorErrorKind::DuplicateAlias, error.kind());
    assert_eq!(Some("file"), error.alias());
}
