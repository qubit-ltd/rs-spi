// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{RegistrationError, RegistrationErrorKind};

#[test]
fn registration_error_exposes_its_kind_and_identifier() {
    let error = RegistrationError::invalid_identifier("Bad Id");

    assert_eq!(RegistrationErrorKind::InvalidIdentifier, error.kind());
    assert_eq!(Some("Bad Id"), error.identifier());
}
