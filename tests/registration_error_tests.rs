// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{RegistrationError, RegistrationErrorKind};

#[test]
fn registration_error_exposes_its_kind_and_conflict_details() {
    let error = RegistrationError::duplicate_selector("en", "english", "spanish");

    assert_eq!(RegistrationErrorKind::DuplicateSelector, error.kind());
    assert_eq!("en", error.selector());
    assert_eq!("english", error.existing_provider());
    assert_eq!("spanish", error.provider());
}
