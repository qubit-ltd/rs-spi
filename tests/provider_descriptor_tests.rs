// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_spi::{ProviderDescriptor, ProviderDescriptorErrorKind, ProviderId};

#[test]
fn descriptor_keeps_typed_metadata() {
    let descriptor = ProviderDescriptor::new(ProviderId::new("file-command").unwrap())
        .with_aliases(["file", "command"])
        .unwrap()
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

#[test]
fn descriptor_rejects_an_alias_that_duplicates_its_canonical_id() {
    let error = ProviderDescriptor::new(ProviderId::new("file-command").unwrap())
        .with_aliases(["file-command"])
        .unwrap_err();

    assert_eq!(ProviderDescriptorErrorKind::AliasMatchesId, error.kind());
    assert_eq!(Some("file-command"), error.alias());
}

#[test]
fn descriptor_reports_invalid_alias_with_position_and_source() {
    let error = ProviderDescriptor::new(ProviderId::new("file-command").expect("valid ID"))
        .with_aliases(["file", "bad alias"])
        .expect_err("invalid alias should fail");

    assert_eq!(ProviderDescriptorErrorKind::InvalidAlias, error.kind());
    assert_eq!(Some(1), error.alias_index());
    assert_eq!(Some("bad alias"), error.alias());
    assert!(Error::source(&error).is_some());
}

#[test]
fn descriptor_distinguishes_duplicate_aliases() {
    let error = ProviderDescriptor::new(ProviderId::new("file-command").expect("valid ID"))
        .with_aliases([" File ", "file"])
        .expect_err("normalized duplicate aliases should fail");

    assert_eq!(ProviderDescriptorErrorKind::DuplicateAlias, error.kind());
    assert_eq!(Some("file"), error.alias());
}
