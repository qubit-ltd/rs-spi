// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::sync::{
    Arc,
    atomic::{
        AtomicUsize,
        Ordering,
    },
};
use std::thread;

use qubit_spi::error::{
    AttemptFailure,
    ProviderError,
    ProviderErrorKind,
    ProviderSelectorError,
    ResolutionError,
};
use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderResolver,
    ProviderSelection,
    ResolutionTermination,
    ServiceProvider,
    ServiceSpec,
};

/// Service interface returned by resolver test providers.
trait Greeting: Send + Sync {
    /// Returns the provider-specific greeting text.
    ///
    /// # Returns
    ///
    /// The stable greeting message.
    fn message(&self) -> &'static str;
}

/// Greeting implementation backed by a static fixture string.
struct StaticGreeting(
    /// Stable message returned by this greeting.
    &'static str,
);

impl Greeting for StaticGreeting {
    /// Returns the fixture message.
    fn message(&self) -> &'static str {
        self.0
    }
}

/// Service family pairing unit configuration with shared greeting handles.
struct GreetingSpec;

impl ServiceSpec for GreetingSpec {
    type Config = ();
    type Output = Arc<dyn Greeting>;
}

/// Provider returning either a fixture greeting or a classified failure.
struct TestProvider(
    /// Configured provider outcome cloned or converted during creation.
    Result<&'static str, ProviderError>,
);

impl ServiceProvider<GreetingSpec> for TestProvider {
    /// Creates the configured greeting outcome.
    ///
    /// # Arguments
    ///
    /// * `_config` - Unused unit configuration.
    ///
    /// # Returns
    ///
    /// A shared greeting when this fixture is configured for success.
    ///
    /// # Errors
    ///
    /// Returns a clone of the configured provider failure.
    fn create(&self, _config: &()) -> Result<Arc<dyn Greeting>, ProviderError> {
        match &self.0 {
            Ok(message) => Ok(Arc::new(StaticGreeting(message))),
            Err(error) => Err(error.clone()),
        }
    }
}

/// Verifies absence fallback and winning-provider attribution in auto order.
#[test]
fn test_on_absence_uses_the_next_automatic_provider_and_reports_the_winner() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("remote")
                    .expect("test provider ID should be valid"),
            )
            .with_priority(20),
            TestProvider(Err(ProviderError::unavailable("disabled"))),
        )
        .expect("remote provider should register");
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("memory")
                    .expect("test provider ID should be valid"),
            )
            .with_priority(10),
            TestProvider(Ok("memory")),
        )
        .expect("memory provider should register");

    let created =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence)
            .create(&ProviderSelection::auto(), &())
            .expect("absence fallback should reach the memory provider");

    assert_eq!("memory", created.provider_id().as_str());
    assert_eq!("memory", created.service().message());
}

/// Verifies that initialization failure stops the absence-only policy.
#[test]
fn test_on_absence_stops_after_an_initialization_failure() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("remote")
                    .expect("test provider ID should be valid"),
            )
            .with_priority(20),
            TestProvider(Err(ProviderError::initialization_failed("broken"))),
        )
        .expect("remote provider should register");
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("memory")
                    .expect("test provider ID should be valid"),
            )
            .with_priority(10),
            TestProvider(Ok("memory")),
        )
        .expect("memory provider should register");

    let result =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence)
            .create(&ProviderSelection::auto(), &());
    let error = match result {
        Ok(_) => panic!("initialization failure must stop fallback"),
        Err(error) => error,
    };

    assert!(matches!(
        &error,
        ResolutionError::NoProviderSucceeded { .. }
    ));
    assert_eq!(
        Some(ResolutionTermination::StoppedByPolicy),
        error.termination(),
    );
    assert_eq!(1, error.attempts().len());
    assert_eq!(
        error.terminal_attempt().map(ToString::to_string),
        error.decisive_attempt().map(ToString::to_string),
    );
    assert!(!error.is_absence());
    assert!(
        error
            .to_string()
            .contains("stopped by fallback policy after 1 attempt"),
    );
    let Some(AttemptFailure::ProviderError {
        error: provider_error,
        ..
    }) = error.decisive_attempt()
    else {
        panic!("policy stop should retain its decisive provider failure");
    };
    assert_eq!(
        ProviderErrorKind::InitializationFailed,
        provider_error.kind(),
    );
}

/// Verifies that a policy-stopped aggregate exposes its decisive attempt.
#[test]
fn test_policy_stop_after_fallback_exposes_its_decisive_source() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("remote")
                    .expect("test provider ID should be valid"),
            )
            .with_priority(20),
            TestProvider(Err(ProviderError::unavailable("disabled"))),
        )
        .expect("remote provider should register");
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("broken")
                    .expect("test provider ID should be valid"),
            )
            .with_priority(10),
            TestProvider(Err(ProviderError::initialization_failed("broken"))),
        )
        .expect("broken provider should register");

    let result =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence)
            .create(&ProviderSelection::auto(), &());
    let error = match result {
        Ok(_) => panic!("initialization failure must stop fallback"),
        Err(error) => error,
    };

    assert_eq!(2, error.attempts().len());
    assert_eq!(
        Some(ResolutionTermination::StoppedByPolicy),
        error.termination(),
    );
    let decisive = error
        .decisive_attempt()
        .expect("policy stop should retain its decisive attempt");
    let source = Error::source(&error)
        .and_then(|source| source.downcast_ref::<AttemptFailure>())
        .expect("policy stop should expose its decisive attempt as the source");
    assert!(std::ptr::eq(decisive, source));
}

/// Verifies resolver accessors, clone behavior, and debug output.
#[test]
fn test_resolver_accessors_clone_and_debug() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("memory")
                    .expect("test provider ID should be valid"),
            ),
            TestProvider(Ok("memory")),
        )
        .expect("memory provider should register");
    let resolver =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAnyError);

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
        cloned
            .registry()
            .provider_ids()
            .next()
            .expect("registry should retain its provider")
            .as_str()
    );
    let debug = format!("{resolver:?}");
    assert!(debug.contains("ProviderResolver"));
    assert!(debug.contains("OnAnyError"));
    assert!(debug.contains("memory"));
}

/// Verifies that the any-error policy continues after initialization failure.
#[test]
fn test_on_any_error_continues_after_initialization_failure() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("remote")
                    .expect("test provider ID should be valid"),
            )
            .with_priority(20),
            TestProvider(Err(ProviderError::initialization_failed("broken"))),
        )
        .expect("remote provider should register");
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("memory")
                    .expect("test provider ID should be valid"),
            )
            .with_priority(10),
            TestProvider(Ok("memory")),
        )
        .expect("memory provider should register");

    let created =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAnyError)
            .create(&ProviderSelection::auto(), &())
            .expect("any-error fallback should reach the memory provider");

    assert_eq!("memory", created.provider_id().as_str());
}

/// Provider counting invocations before returning an absence-class failure.
struct CountingProvider {
    /// Shared invocation counter used to detect duplicate provider attempts.
    attempts: Arc<AtomicUsize>,
}

impl ServiceProvider<GreetingSpec> for CountingProvider {
    /// Counts one invocation and returns an unavailable-provider failure.
    ///
    /// # Arguments
    ///
    /// * `_config` - Unused unit configuration.
    ///
    /// # Returns
    ///
    /// This fixture never returns a service.
    ///
    /// # Errors
    ///
    /// Always returns an unavailable-provider error.
    fn create(&self, _config: &()) -> Result<Arc<dyn Greeting>, ProviderError> {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        Err(ProviderError::unavailable("not ready"))
    }
}

/// Verifies unknown-attempt recording and alias-based provider deduplication.
#[test]
fn test_chain_records_unknown_selectors_and_deduplicates_provider_aliases() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("remote")
                    .expect("test provider ID should be valid"),
            )
            .with_aliases(["cloud"])
            .expect("test alias should be valid"),
            CountingProvider {
                attempts: Arc::clone(&attempts),
            },
        )
        .expect("counting provider should register");
    let resolver =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
    let selection = ProviderSelection::chain(["missing", "cloud", "remote"])
        .expect("test selector chain should be valid");

    let error = match resolver.create(&selection, &()) {
        Ok(_) => panic!("the exhausted chain must fail"),
        Err(error) => error,
    };

    assert_eq!(1, attempts.load(Ordering::SeqCst));
    assert_eq!(Some(ResolutionTermination::Exhausted), error.termination(),);
    assert!(error.is_absence());
    let ResolutionError::NoProviderSucceeded {
        attempts: failures, ..
    } = error
    else {
        panic!("an exhausted chain should produce an aggregate error");
    };
    let [
        AttemptFailure::UnknownProvider {
            requested_selector, ..
        },
        AttemptFailure::ProviderError { provider_id, .. },
    ] = failures.as_ref()
    else {
        panic!("the chain should retain one lookup and one provider failure");
    };
    assert_eq!("missing", requested_selector.as_str());
    assert_eq!("remote", provider_id.as_str());
}

/// Verifies that named selection never invokes another registered provider.
#[test]
fn test_named_selection_never_falls_back() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("remote")
                    .expect("test provider ID should be valid"),
            ),
            TestProvider(Err(ProviderError::unavailable("disabled"))),
        )
        .expect("remote provider should register");
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("memory")
                    .expect("test provider ID should be valid"),
            ),
            TestProvider(Ok("memory")),
        )
        .expect("memory provider should register");
    let resolver =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAnyError);

    let selection = ProviderSelection::named("remote")
        .expect("test named selector should be valid");
    let error = match resolver.create(&selection, &()) {
        Ok(_) => panic!("named selection must report its provider failure"),
        Err(error) => error,
    };

    assert!(error.decisive_attempt().is_some());
    let ResolutionError::NoProviderSucceeded { attempts, .. } = error else {
        panic!("named provider failure should produce an aggregate error");
    };
    let [AttemptFailure::ProviderError { provider_id, .. }] = attempts.as_ref()
    else {
        panic!("named resolution should retain one provider failure");
    };
    assert_eq!("remote", provider_id.as_str());
}

/// Verifies fallback decisions for every provider error classification.
#[test]
fn test_fallback_policies_cover_every_provider_error_classification() {
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

/// Verifies concurrent service creation through cloned immutable resolvers.
#[test]
fn test_cloned_resolver_supports_concurrent_service_creation() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("memory")
                    .expect("test provider ID should be valid"),
            ),
            TestProvider(Ok("memory")),
        )
        .expect("memory provider should register");
    let resolver =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);

    let threads = (0..8)
        .map(|_| {
            let resolver = resolver.clone();
            thread::spawn(move || {
                resolver
                    .create(
                        &ProviderSelection::named("memory")
                            .expect("test named selector should be valid"),
                        &(),
                    )
                    .expect("registered provider should create a greeting")
                    .service()
                    .message()
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        assert_eq!(
            "memory",
            thread.join().expect("resolver thread should not panic"),
        );
    }
}

/// Verifies preservation of raw invalid input in named resolution errors.
#[test]
fn test_raw_named_resolution_preserves_invalid_selector_input() {
    let resolver = ProviderResolver::<GreetingSpec>::new(
        ProviderRegistry::default(),
        FallbackPolicy::OnAbsence,
    );
    let error = match resolver.create_named(" Bad Selector ", &()) {
        Ok(_) => panic!("invalid raw selector should fail"),
        Err(error) => error,
    };

    let ResolutionError::InvalidSelector {
        selector_index,
        source,
        ..
    } = &error
    else {
        panic!("invalid input should produce an invalid-selector error");
    };
    assert_eq!(None, *selector_index);
    assert_eq!(" Bad Selector ", source.input());
    assert!(matches!(source, ProviderSelectorError::Invalid { .. }));
    assert!(error.attempts().is_empty());
    assert!(error.termination().is_none());
    assert!(error.terminal_attempt().is_none());
    assert!(!error.is_absence());
    assert!(Error::source(&error).is_some());
    assert_eq!(
        "invalid provider selector \" Bad Selector \"",
        error.to_string(),
    );
}

/// Verifies that a valid but unknown raw named selector retains typed context.
#[test]
fn test_raw_named_resolution_reports_an_unknown_provider() {
    let resolver = ProviderResolver::<GreetingSpec>::new(
        ProviderRegistry::default(),
        FallbackPolicy::OnAbsence,
    );
    let error = match resolver.create_named(" Missing ", &()) {
        Ok(_) => panic!("unknown raw selector should fail"),
        Err(error) => error,
    };

    let ResolutionError::UnknownProvider { selector, .. } = &error else {
        panic!("unknown input should produce an unknown-provider error");
    };
    assert_eq!("missing", selector.as_str());
    assert!(Error::source(&error).is_none());
    assert_eq!("unknown provider: missing", error.to_string());
}

/// Verifies chain selector positions and rejection of an empty raw chain.
#[test]
fn test_raw_chain_reports_invalid_selector_position_and_empty_input() {
    let resolver = ProviderResolver::<GreetingSpec>::new(
        ProviderRegistry::default(),
        FallbackPolicy::OnAbsence,
    );
    let invalid = match resolver.create_chain(["valid", "bad selector"], &()) {
        Ok(_) => panic!("invalid raw chain selector should fail"),
        Err(error) => error,
    };
    let ResolutionError::InvalidSelector {
        selector_index,
        source,
        ..
    } = &invalid
    else {
        panic!("invalid chain input should report its selector position");
    };
    assert_eq!(Some(1), *selector_index);
    assert_eq!("bad selector", source.input());
    assert!(matches!(source, ProviderSelectorError::Invalid { .. }));
    assert_eq!(
        "invalid provider selector at chain index 1: \"bad selector\"",
        invalid.to_string(),
    );

    let empty = match resolver.create_chain(Vec::<&str>::new(), &()) {
        Ok(_) => panic!("empty raw chain should fail"),
        Err(error) => error,
    };
    assert!(matches!(&empty, ResolutionError::EmptySelection));
    assert!(!empty.is_absence());
    assert!(Error::source(&empty).is_none());
    assert_eq!(
        "provider selection chain must not be empty",
        empty.to_string(),
    );
}

/// Verifies that automatic resolution distinguishes an empty registry.
#[test]
fn test_automatic_resolution_distinguishes_an_empty_registry() {
    let resolver = ProviderResolver::<GreetingSpec>::new(
        ProviderRegistry::default(),
        FallbackPolicy::OnAbsence,
    );
    let error = match resolver.create_auto(&()) {
        Ok(_) => panic!("empty registry should fail"),
        Err(error) => error,
    };

    assert!(matches!(&error, ResolutionError::EmptyRegistry));
    assert!(error.decisive_attempt().is_none());
    assert!(!error.is_absence());
    assert!(Error::source(&error).is_none());
    assert!(error.to_string().contains("empty"));
}

/// Verifies aggregate display order and retention of attempt diagnostics.
#[test]
fn test_aggregate_resolution_display_contains_ordered_attempt_diagnostics() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    for (id, reason) in [
        ("first", "first unavailable"),
        ("second", "second unsupported"),
    ] {
        let error = if id == "first" {
            ProviderError::unavailable(reason)
        } else {
            ProviderError::unsupported(reason)
        };
        builder
            .register(
                ProviderDescriptor::new(
                    ProviderId::new(id).expect("valid provider ID"),
                ),
                TestProvider(Err(error)),
            )
            .expect("unique provider should register");
    }
    let resolver =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
    let error = match resolver.create_chain(["first", "second"], &()) {
        Ok(_) => panic!("exhausted chain should fail"),
        Err(error) => error,
    };
    let message = error.to_string();

    assert!(matches!(
        &error,
        ResolutionError::NoProviderSucceeded { .. }
    ));
    assert_eq!(Some(ResolutionTermination::Exhausted), error.termination(),);
    assert_eq!(2, error.attempts().len());
    assert!(error.terminal_attempt().is_some());
    assert!(error.decisive_attempt().is_none());
    assert!(error.is_absence());
    assert!(Error::source(&error).is_none());

    let first = message.find("first unavailable").expect("first reason");
    let second = message.find("second unsupported").expect("second reason");
    assert!(message.contains("first"));
    assert!(message.contains("second"));
    assert!(
        first < second,
        "attempts should be formatted in encounter order"
    );
}

/// Verifies raw chains return the first provider that creates a service.
#[test]
fn test_raw_chain_returns_the_first_successful_provider() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("memory")
                    .expect("test provider ID should be valid"),
            ),
            TestProvider(Ok("memory")),
        )
        .expect("memory provider should register");
    let resolver =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);

    let created = resolver
        .create_chain(["missing", "memory"], &())
        .expect("raw chain should reach the registered provider");

    assert_eq!("memory", created.provider_id().as_str());
    assert_eq!("memory", created.service().message());
}

/// Verifies raw chains stop on failures disallowed by the fallback policy.
#[test]
fn test_raw_chain_stops_on_a_disallowed_provider_failure() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("remote")
                    .expect("test provider ID should be valid"),
            ),
            TestProvider(Err(ProviderError::initialization_failed("broken"))),
        )
        .expect("remote provider should register");
    let resolver =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);

    let error = match resolver.create_chain(["remote", "missing"], &()) {
        Ok(_) => panic!("initialization failure should stop the raw chain"),
        Err(error) => error,
    };

    let ResolutionError::NoProviderSucceeded { attempts, .. } = error else {
        panic!("disallowed provider failure should produce an aggregate error");
    };
    assert_eq!(1, attempts.len());
}

/// Verifies automatic resolution aggregates all permitted failures.
#[test]
fn test_automatic_resolution_aggregates_all_permitted_failures() {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    for id in ["first", "second"] {
        builder
            .register(
                ProviderDescriptor::new(
                    ProviderId::new(id)
                        .expect("test provider ID should be valid"),
                ),
                TestProvider(Err(ProviderError::unavailable("offline"))),
            )
            .expect("unique provider should register");
    }
    let resolver =
        ProviderResolver::new(builder.build(), FallbackPolicy::OnAnyError);

    let error = match resolver.create_auto(&()) {
        Ok(_) => panic!("all automatic providers should fail"),
        Err(error) => error,
    };

    assert_eq!(Some(ResolutionTermination::Exhausted), error.termination(),);
    assert_eq!(2, error.attempts().len());
}

/// Reports whether automatic resolution reaches the second provider.
///
/// # Arguments
///
/// * `first_error` - Failure returned by the highest-priority provider.
/// * `policy` - Fallback policy under test.
///
/// # Returns
///
/// `true` when the fallback provider creates the service.
fn automatic_reaches_fallback(
    first_error: ProviderError,
    policy: FallbackPolicy,
) -> bool {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("first")
                    .expect("test provider ID should be valid"),
            )
            .with_priority(20),
            TestProvider(Err(first_error)),
        )
        .expect("first provider should register");
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("fallback")
                    .expect("test provider ID should be valid"),
            )
            .with_priority(10),
            TestProvider(Ok("fallback")),
        )
        .expect("fallback provider should register");

    ProviderResolver::new(builder.build(), policy)
        .create(&ProviderSelection::auto(), &())
        .is_ok()
}
