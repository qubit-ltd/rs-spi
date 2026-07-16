// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for stable registration-error classifications.

use qubit_spi::error::RegistrationErrorKind;

#[test]
fn test_registration_error_kind_is_public() {
    let kind = RegistrationErrorKind::DuplicateSelector;
    assert_eq!(RegistrationErrorKind::DuplicateSelector, kind);
}
