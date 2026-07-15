// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::thread;

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

#[test]
fn resolver_exposes_immutable_configuration_and_is_cloneable_and_debuggable() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("memory").unwrap()),
            TestProvider(Ok("memory")),
        )
        .unwrap();
    let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAnyError);

    assert_eq!(FallbackPolicy::OnAnyError, resolver.fallback_policy());
    assert_eq!(
        vec!["memory"],
        resolver
            .registry()
            .provider_ids()
            .map(ProviderId::as_str)
            .collect::<Vec<_>>(),
    );
    let cloned = resolver.clone();
    assert_eq!(
        "memory",
        cloned.registry().provider_ids().next().unwrap().as_str()
    );
    let debug = format!("{resolver:?}");
    assert!(debug.contains("ProviderResolver"));
    assert!(debug.contains("OnAnyError"));
    assert!(debug.contains("memory"));
}

#[test]
fn on_any_error_continues_after_initialization_failure() {
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

    let created = ProviderResolver::new(builder.build(), FallbackPolicy::OnAnyError)
        .create(&ProviderSelection::Auto, &())
        .unwrap();

    assert_eq!("memory", created.provider_id().as_str());
}

struct CountingProvider {
    attempts: Arc<AtomicUsize>,
}

impl ServiceProvider<GreetingSpec> for CountingProvider {
    fn create(&self, _config: &()) -> Result<Arc<dyn Greeting>, ProviderError> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        Err(ProviderError::unavailable("not ready"))
    }
}

#[test]
fn chain_records_unknown_selectors_and_deduplicates_provider_aliases() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("remote").unwrap())
                .with_aliases(["cloud"])
                .unwrap(),
            CountingProvider {
                attempts: Arc::clone(&attempts),
            },
        )
        .unwrap();
    let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
    let selection = ProviderSelection::chain(["missing", "cloud", "remote"]).unwrap();

    let error = match resolver.create(&selection, &()) {
        Ok(_) => panic!("the exhausted chain must fail"),
        Err(error) => error,
    };

    assert_eq!(1, attempts.load(Ordering::SeqCst));
    assert_eq!(2, error.attempts().len());
    assert_eq!("unknown provider: missing", error.attempts()[0].reason());
    assert_eq!(
        Some("remote"),
        error.attempts()[1].provider_id().map(ProviderId::as_str)
    );
}

#[test]
fn named_selection_never_falls_back() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("remote").unwrap()),
            TestProvider(Err(ProviderError::unavailable("disabled"))),
        )
        .unwrap();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("memory").unwrap()),
            TestProvider(Ok("memory")),
        )
        .unwrap();
    let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAnyError);

    let error = match resolver.create(&ProviderSelection::named("remote").unwrap(), &()) {
        Ok(_) => panic!("named selection must report its provider failure"),
        Err(error) => error,
    };

    assert_eq!(1, error.attempts().len());
    assert_eq!(
        Some("remote"),
        error.attempts()[0].provider_id().map(ProviderId::as_str)
    );
}

#[test]
fn fallback_policies_cover_every_provider_error_classification() {
    let on_absence = [
        (ProviderError::unsupported("unsupported"), true),
        (ProviderError::unavailable("unavailable"), true),
        (
            ProviderError::invalid_configuration("invalid configuration"),
            false,
        ),
        (
            ProviderError::initialization_failed("initialization failed"),
            false,
        ),
    ];
    for (error, expected_fallback) in on_absence {
        assert_eq!(
            expected_fallback,
            automatic_reaches_fallback(error, FallbackPolicy::OnAbsence),
        );
    }

    let on_any_error = [
        ProviderError::unsupported("unsupported"),
        ProviderError::unavailable("unavailable"),
        ProviderError::invalid_configuration("invalid configuration"),
        ProviderError::initialization_failed("initialization failed"),
    ];
    for error in on_any_error {
        assert!(automatic_reaches_fallback(
            error,
            FallbackPolicy::OnAnyError,
        ));
    }
}

#[test]
fn cloned_resolver_supports_concurrent_service_creation() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("memory").unwrap()),
            TestProvider(Ok("memory")),
        )
        .unwrap();
    let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);

    let threads = (0..8)
        .map(|_| {
            let resolver = resolver.clone();
            thread::spawn(move || {
                resolver
                    .create(&ProviderSelection::named("memory").unwrap(), &())
                    .unwrap()
                    .service()
                    .message()
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        assert_eq!("memory", thread.join().unwrap());
    }
}

fn automatic_reaches_fallback(first_error: ProviderError, policy: FallbackPolicy) -> bool {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("first").unwrap()).with_priority(20),
            TestProvider(Err(first_error)),
        )
        .unwrap();
    builder
        .register(
            ProviderDescriptor::new(ProviderId::new("fallback").unwrap()).with_priority(10),
            TestProvider(Ok("fallback")),
        )
        .unwrap();

    ProviderResolver::new(builder.build(), policy)
        .create(&ProviderSelection::Auto, &())
        .is_ok()
}
