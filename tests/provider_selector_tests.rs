// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderSelector;

#[test]
fn selector_normalizes_configuration_input() {
    assert_eq!(
        "git+ssh",
        ProviderSelector::parse(" Git+SSH ").unwrap().as_str(),
    );
}
