// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::{
    error::Error,
    fmt,
};

use qubit_spi::error::{
    ProviderFailure,
    ProviderFailureKind,
};

/// Non-cloneable domain error used to prove provider failures retain ownership.
#[derive(Debug)]
struct TestError(&'static str);

impl fmt::Display for TestError {
    /// Formats the diagnostic message.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination formatter.
    ///
    /// # Returns
    ///
    /// The formatter result.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl Error for TestError {}

/// Verifies typed provider failures preserve a non-cloneable domain error.
#[test]
fn test_provider_failure_into_parts_preserves_non_clone_error() {
    let failure = ProviderFailure::invalid_configuration(TestError("bad"));
    let (kind, error) = failure.into_parts();

    assert_eq!(ProviderFailureKind::InvalidConfiguration, kind);
    assert_eq!("bad", error.0);
}
