// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::provider_descriptor;

/// Verifies that the static-descriptor macro preserves validated metadata.
#[test]
fn test_provider_descriptor_macro_builds_validated_static_metadata() {
    let descriptor = provider_descriptor!(
        "file-command",
        aliases: ["file", "command"],
        priority: 20,
    );

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

/// Verifies that macro support constructs descriptors from validated literals.
#[test]
fn test_static_literal_descriptor_constructor_builds_metadata() {
    let descriptor = ProviderDescriptor::__from_static_literals("file-command", &["file", "command"], 20);

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
fn test_static_literal_validation_rejects_invalid_and_conflicting_metadata() {
    assert!(ProviderDescriptor::__are_valid_static_literals(
        "provider",
        &["alias", "other"],
    ));
    assert!(!ProviderDescriptor::__are_valid_static_literals("Provider", &[],));
    assert!(!ProviderDescriptor::__are_valid_static_literals("provider", &["Alias"],));
    assert!(!ProviderDescriptor::__are_valid_static_literals(
        "provider",
        &["provider"],
    ));
    assert!(!ProviderDescriptor::__are_valid_static_literals(
        "provider",
        &["alias", "alias"],
    ));
}

/// Verifies that a descriptor retains typed ID, alias, and priority metadata.
#[test]
fn test_descriptor_keeps_typed_metadata() {
    let descriptor =
        ProviderDescriptor::new(ProviderId::new("file-command").expect("test provider ID should be valid"))
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
