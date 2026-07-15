// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::Arc;

use qubit_spi::{
    FallbackPolicy, ProviderDescriptor, ProviderError, ProviderId, ProviderRegistry,
    ProviderResolver, ProviderSelection, ServiceProvider, ServiceSpec,
};

trait Greeting: Send + Sync {
    fn message(&self) -> &'static str;
}

struct StaticGreeting(&'static str);

impl Greeting for StaticGreeting {
    fn message(&self) -> &'static str {
        self.0
    }
}

struct GreetingSpec;

impl ServiceSpec for GreetingSpec {
    type Config = ();
    type Output = Arc<dyn Greeting>;
}

struct TestProvider(Result<&'static str, ProviderError>);

impl ServiceProvider<GreetingSpec> for TestProvider {
    fn create(&self, _config: &()) -> Result<Arc<dyn Greeting>, ProviderError> {
        match &self.0 {
            Ok(message) => Ok(Arc::new(StaticGreeting(message))),
            Err(error) => Err(error.clone()),
        }
    }
}

#[test]
fn on_absence_uses_the_next_automatic_provider_and_reports_the_winner() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("remote").unwrap()).with_priority(20),
            TestProvider(Err(ProviderError::unavailable("disabled"))),
        )
        .unwrap();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("memory").unwrap()).with_priority(10),
            TestProvider(Ok("memory")),
        )
        .unwrap();

    let created = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence)
        .create(&ProviderSelection::Auto, &())
        .unwrap();

    assert_eq!("memory", created.provider_id().as_str());
    assert_eq!("memory", created.service().message());
}

#[test]
fn on_absence_stops_after_an_initialization_failure() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("remote").unwrap()).with_priority(20),
            TestProvider(Err(ProviderError::initialization_failed("broken"))),
        )
        .unwrap();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("memory").unwrap()).with_priority(10),
            TestProvider(Ok("memory")),
        )
        .unwrap();

    let result = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence)
        .create(&ProviderSelection::Auto, &());
    let error = match result {
        Ok(_) => panic!("initialization failure must stop fallback"),
        Err(error) => error,
    };

    assert_eq!(1, error.attempts().len());
}
