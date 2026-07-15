// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::Arc;

use qubit_spi::{
    ProviderError,
    ServiceProvider,
    ServiceSpec,
};

/// Minimal service interface returned by the test service specification.
trait Counter: Send + Sync {
    /// Returns the stable counter value.
    ///
    /// # Returns
    ///
    /// The counter's current value.
    fn value(&self) -> u8;
}

/// Counter implementation returning a stable value.
struct StaticCounter;

impl Counter for StaticCounter {
    /// Returns the test value seven.
    fn value(&self) -> u8 {
        7
    }
}

/// Service family pairing unit configuration with a shared counter handle.
struct CounterSpec;

impl ServiceSpec for CounterSpec {
    type Config = ();
    type Output = Arc<dyn Counter>;
}

/// Provider constructing the static counter implementation.
struct CounterProvider;

impl ServiceProvider<CounterSpec> for CounterProvider {
    /// Creates a shared static counter.
    ///
    /// # Arguments
    ///
    /// * `_config` - Unused unit configuration.
    ///
    /// # Returns
    ///
    /// A shared counter returning seven.
    fn create(&self, _config: &()) -> Result<Arc<dyn Counter>, ProviderError> {
        Ok(Arc::new(StaticCounter))
    }
}

/// Verifies that a provider returns the output handle selected by its spec.
#[test]
fn test_provider_creates_the_handle_selected_by_the_spec() {
    assert_eq!(
        7,
        CounterProvider
            .create(&())
            .expect("test provider should create its counter")
            .value(),
    );
}
