// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for stable provider-descriptor-error classifications.

use qubit_spi::error::ProviderDescriptorErrorKind;

#[test]
fn test_provider_descriptor_error_kind_values_are_distinct() {
    assert_ne!(
        ProviderDescriptorErrorKind::InvalidAlias,
        ProviderDescriptorErrorKind::DuplicateAlias,
    );
    assert_ne!(
        ProviderDescriptorErrorKind::DuplicateAlias,
        ProviderDescriptorErrorKind::AliasMatchesId,
    );
}
