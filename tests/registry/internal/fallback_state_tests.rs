// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::FallbackPolicy;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::ProviderRegistry;
use qubit_spi::ProviderSelection;

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_error::TestProviderFailure;
use crate::common::test_provider_definition::define_provider;

/// Verifies resolver fallback state advances after an absence failure.
#[test]
fn test_fallback_state_advances_to_the_next_candidate() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("missing").expect("static ID should be valid")),
            ConfigurableProvider::failure(TestProviderFailure::unavailable("provider is absent")),
        ))
        .expect("first provider should register");
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("ready").expect("static ID should be valid")),
            ConfigurableProvider::success("ready"),
        ))
        .expect("second provider should register");
    let selection = ProviderSelection::chain(["missing", "ready"])
        .expect("static chain should be valid")
        .with_fallback_policy(FallbackPolicy::OnAbsence);

    let output = registry
        .resolve_selected(&selection)
        .expect("chain should resolve")
        .create_configured(&String::new())
        .expect("fallback candidate should create the service");

    assert_eq!("ready", output);
}
