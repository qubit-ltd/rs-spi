// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::Arc;

use qubit_spi::{ProviderError, ServiceProvider, ServiceSpec};

trait Counter: Send + Sync {
    fn value(&self) -> u8;
}

struct StaticCounter;

impl Counter for StaticCounter {
    fn value(&self) -> u8 {
        7
    }
}

struct CounterSpec;

impl ServiceSpec for CounterSpec {
    type Config = ();
    type Output = Arc<dyn Counter>;
}

struct CounterProvider;

impl ServiceProvider<CounterSpec> for CounterProvider {
    fn create(&self, _config: &()) -> Result<Arc<dyn Counter>, ProviderError> {
        Ok(Arc::new(StaticCounter))
    }
}

#[test]
fn provider_creates_the_handle_selected_by_the_spec() {
    assert_eq!(7, CounterProvider.create(&()).unwrap().value());
}
