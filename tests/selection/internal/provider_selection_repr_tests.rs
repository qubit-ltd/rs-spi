// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::ProviderResolutionError;
use qubit_spi::{
    ProviderRegistry,
    ProviderSelection,
};

use crate::common::string_spec::StringSpec;

/// Verifies the private strict-chain representation preserves missing entries.
#[test]
fn test_provider_selection_repr_rejects_unknown_chain_entries() {
    let registry = ProviderRegistry::<StringSpec>::default();
    let selection = ProviderSelection::chain(["unknown"])
        .expect("static chain should be valid");

    assert!(matches!(
        registry.resolve_selected(&selection),
        Err(ProviderResolutionError::UnknownProviders { selectors, .. })
            if selectors[0].as_str() == "unknown"
    ));
}
