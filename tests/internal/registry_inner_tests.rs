// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderRegistry;

use crate::common::string_spec::StringSpec;

/// Verifies a new Registry exposes the empty internal state consistently.
#[test]
fn test_registry_inner_starts_empty() {
    let registry = ProviderRegistry::<StringSpec>::default();

    assert!(registry.is_empty());
    assert_eq!(0, registry.len());
    assert!(registry.provider_ids().is_empty());
    assert!(registry.descriptors().is_empty());
}
