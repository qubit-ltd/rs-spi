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

/// Invalid aliases stop consuming a caller's iterator immediately.
#[test]
fn test_with_aliases_stops_consuming_after_first_invalid_input() {
    use std::cell::Cell;

    use qubit_spi::error::ProviderDescriptorError;
    let consumed = Cell::new(0);
    let aliases = ["invalid alias", "valid", "unused"]
        .into_iter()
        .inspect(|_| consumed.set(consumed.get() + 1));
    let error = ProviderDescriptor::new(ProviderId::new("provider").expect("valid ID"))
        .with_aliases(aliases)
        .expect_err("first alias is invalid");
    assert!(matches!(
        error,
        ProviderDescriptorError::InvalidAlias { alias_index: 0, .. }
    ));
    assert_eq!(1, consumed.get());
}

/// Successful alias validation consumes every item and retains encounter order.
#[test]
fn test_with_aliases_consumes_all_successful_inputs_in_order() {
    use std::cell::Cell;
    let consumed = Cell::new(0);
    let aliases = [" FIRST ", "Second", "third"]
        .into_iter()
        .inspect(|_| consumed.set(consumed.get() + 1));
    let descriptor = ProviderDescriptor::new(ProviderId::new("provider").expect("valid ID"))
        .with_aliases(aliases)
        .expect("all aliases normalize successfully");
    assert_eq!(3, consumed.get());
    assert_eq!(
        vec!["first", "second", "third"],
        descriptor
            .aliases()
            .iter()
            .map(|alias| alias.as_str())
            .collect::<Vec<_>>()
    );
}
