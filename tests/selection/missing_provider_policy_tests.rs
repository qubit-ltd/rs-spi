// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::MissingProviderPolicy;

/// Verifies that chained selection rejects missing providers by default.
#[test]
fn test_missing_provider_policy_defaults_to_reject() {
    assert_eq!(
        MissingProviderPolicy::Reject,
        MissingProviderPolicy::default(),
    );
}

/// Verifies that the missing-provider policies remain distinct value types.
#[test]
fn test_missing_provider_policy_values_are_copyable_and_distinct() {
    let policy = MissingProviderPolicy::Ignore;
    let copied = policy;

    assert_eq!(MissingProviderPolicy::Ignore, copied);
    assert_ne!(MissingProviderPolicy::Reject, copied);
    assert!(format!("{copied:?}").contains("Ignore"));
}
