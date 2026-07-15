// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{ProviderDescriptor, ProviderId, RegistrationErrorKind};

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

    assert_eq!(RegistrationErrorKind::DuplicateSelector, error.kind());
}
