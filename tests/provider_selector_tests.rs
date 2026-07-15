// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{ProviderSelector, ProviderSelectorErrorKind};

#[test]
fn selector_normalizes_configuration_input() {
    assert_eq!(
        "git+ssh",
        ProviderSelector::parse(" Git+SSH ").unwrap().as_str(),
    );
}

#[test]
fn selector_errors_preserve_raw_and_normalized_input() {
    let empty = ProviderSelector::parse("  ").expect_err("blank selector should fail");
    assert_eq!(ProviderSelectorErrorKind::Empty, empty.kind());
    assert_eq!("  ", empty.input());
    assert_eq!(None, empty.normalized());

    let invalid = ProviderSelector::parse(" Bad Selector ")
        .expect_err("selector containing a space should fail");
    assert_eq!(ProviderSelectorErrorKind::Invalid, invalid.kind());
    assert_eq!(" Bad Selector ", invalid.input());
    assert_eq!(Some("bad selector"), invalid.normalized());
}
