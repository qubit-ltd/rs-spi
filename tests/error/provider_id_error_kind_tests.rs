// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for stable provider-ID-error classifications.

use qubit_spi::error::ProviderIdErrorKind;

#[test]
fn test_provider_id_error_kind_values_are_distinct() {
    assert_ne!(
        ProviderIdErrorKind::Empty,
        ProviderIdErrorKind::NonCanonical,
    );
}
