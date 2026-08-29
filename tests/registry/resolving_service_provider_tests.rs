// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;

use qubit_spi::FallbackPolicy;
use qubit_spi::ProviderCreationTermination;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::ProviderRegistry;
use qubit_spi::ProviderSelection;
use qubit_spi::ResolvingServiceProvider;
use qubit_spi::ServiceProvider;
use qubit_spi::error::ProviderFailure;
use qubit_spi::error::ProviderResolutionError;

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_error::TestError;
use crate::common::test_error::TestProviderFailure;
use crate::common::test_provider_definition::define_provider;

mod inherent_api_tests {
    use qubit_spi::ProviderRegistry;
    use qubit_spi::ProviderSelection;

    use super::register_provider;
    use crate::common::configurable_provider::ConfigurableProvider;
    use crate::common::string_spec::StringSpec;

    /// Verifies creation methods remain callable without importing the trait.
    #[test]
    fn test_resolving_provider_exposes_inherent_creation_methods() {
        let registry = ProviderRegistry::<StringSpec>::default();
        register_provider(&registry, "echo", &[], 0, ConfigurableProvider::echo());
        let provider = registry
            .resolve_selected(&ProviderSelection::auto())
            .expect("automatic selection should resolve");

        assert_eq!(
            "explicit",
            provider
                .create_configured(&"explicit".to_owned())
                .expect("explicit creation should succeed"),
        );
        assert_eq!(
            String::default(),
            provider.create().expect("default creation should succeed"),
        );
    }
}

/// Verifies that successful creation returns only the requested service value.
#[test]
fn test_resolving_provider_returns_service_output_directly() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "memory", &[], 0, ConfigurableProvider::success("service"));

    let provider: ResolvingServiceProvider<StringSpec> = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve");
    let debug = format!("{provider:?}");

    assert_eq!(
        "service",
        provider
            .create_configured(&String::new())
            .expect("provider should create its service"),
    );
    assert!(debug.contains("ResolvingServiceProvider"));
    assert!(debug.contains("memory"));
    assert!(debug.contains("OnAbsence"));
}

/// Verifies that explicit service configuration reaches the provider unchanged.
#[test]
fn test_resolving_provider_passes_explicit_config_unchanged() {
    let seen_config = Arc::new(Mutex::new(None));
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "echo",
        &[],
        0,
        ConfigurableProvider::echo().with_seen_config(Arc::clone(&seen_config)),
    );
    let provider = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve");

    let output = provider
        .create_configured(&"explicit".to_owned())
        .expect("echo provider should succeed");

    assert_eq!("explicit", output);
    let seen_config = seen_config.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(Some("explicit"), seen_config.as_deref());
}

/// Verifies that resolving providers use the service config default.
#[test]
fn test_resolving_provider_uses_default_config() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "echo", &[], 0, ConfigurableProvider::echo());
    let provider = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve");

    assert_eq!(
        String::default(),
        provider.create().expect("default creation should succeed"),
    );
}

/// Verifies provider panics propagate without attempting fallback candidates.
#[test]
fn test_resolver_propagates_provider_panic_without_trying_fallback() {
    let fallback_calls = Arc::new(AtomicUsize::new(0));
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "panicking", &[], 20, PanickingProvider);
    register_provider(
        &registry,
        "fallback",
        &[],
        10,
        ConfigurableProvider::success("fallback").with_calls(Arc::clone(&fallback_calls)),
    );
    let resolver = registry
        .resolve_selected(&ProviderSelection::auto().with_fallback_policy(FallbackPolicy::OnAnyError))
        .expect("automatic selection should resolve");

    let result = catch_unwind(AssertUnwindSafe(|| resolver.create()));

    assert!(result.is_err(), "provider panic should propagate");
    assert_eq!(0, fallback_calls.load(Ordering::SeqCst));
}

/// Verifies that a named alias selects exactly its owning provider.
#[test]
fn test_registry_resolves_named_alias_to_one_candidate() {
    let unrelated_calls = Arc::new(AtomicUsize::new(0));
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "english", &["en"], 0, ConfigurableProvider::success("hello"));
    register_provider(
        &registry,
        "unrelated",
        &[],
        100,
        ConfigurableProvider::success("other").with_calls(Arc::clone(&unrelated_calls)),
    );
    let selection = ProviderSelection::named("EN").expect("test selector should be valid");

    let output = registry
        .resolve_selected(&selection)
        .expect("alias should resolve")
        .create()
        .expect("selected provider should succeed");

    assert_eq!("hello", output);
    assert_eq!(0, unrelated_calls.load(Ordering::SeqCst));
}

/// Verifies that strict chains reject every unknown selector before creation.
#[test]
fn test_registry_rejects_partially_unknown_strict_chain() {
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "known",
        &[],
        0,
        ConfigurableProvider::success("known").with_calls(Arc::clone(&calls)),
    );
    let selection = ProviderSelection::chain(["missing", "known"]).expect("strict chain should parse");

    let error = registry
        .resolve_selected(&selection)
        .expect_err("strict chain should reject every unknown selector");

    assert!(matches!(
        error,
        ProviderResolutionError::UnknownProviders { ref selectors, .. }
            if selectors
                .iter()
                .map(|selector| selector.as_str())
                .eq(["missing"])
    ));
    assert_eq!(0, calls.load(Ordering::SeqCst));
}

/// Verifies explicit missing-selector tolerance, chain order, and
/// deduplication.
#[test]
fn test_registry_allows_explicitly_missing_chain_entries_and_deduplicates() {
    let first_calls = Arc::new(AtomicUsize::new(0));
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "first",
        &["one"],
        0,
        ConfigurableProvider::failure(TestProviderFailure::unavailable("offline")).with_calls(Arc::clone(&first_calls)),
    );
    register_provider(&registry, "second", &[], 0, ConfigurableProvider::success("second"));
    let selection = ProviderSelection::chain_allowing_missing(["missing", "one", "first", "second"])
        .expect("optional chain should parse")
        .with_fallback_policy(FallbackPolicy::OnAnyError);

    let output = registry
        .resolve_selected(&selection)
        .expect("chain should resolve known candidates")
        .create()
        .expect("second provider should succeed");

    assert_eq!("second", output);
    assert_eq!(1, first_calls.load(Ordering::SeqCst));
}

/// Verifies automatic ordering by descending priority and canonical ID.
#[test]
fn test_registry_resolves_auto_by_priority_then_id() {
    let registry = ProviderRegistry::<StringSpec>::default();
    for (id, priority) in [("zulu", 10), ("beta", 20), ("alpha", 20)] {
        register_provider(&registry, id, &[], priority, ConfigurableProvider::success(id));
    }

    let output = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve")
        .create()
        .expect("highest-ranked provider should succeed");

    assert_eq!("alpha", output);
}

/// Verifies automatic fallback attempts retain the complete ranked order.
#[test]
fn test_registry_auto_fallback_attempts_follow_ranked_order() {
    let registry = ProviderRegistry::<StringSpec>::default();
    for (id, priority) in [("zulu", 10), ("beta", 20), ("alpha", 20)] {
        register_provider(
            &registry,
            id,
            &[],
            priority,
            ConfigurableProvider::failure(TestProviderFailure::unavailable(format!("{id} unavailable"))),
        );
    }
    let selection = ProviderSelection::auto().with_fallback_policy(FallbackPolicy::OnAnyError);

    let error = registry
        .resolve_selected(&selection)
        .expect("automatic selection should resolve")
        .create()
        .expect_err("all providers should fail");

    assert_eq!(
        vec!["alpha", "beta", "zulu"],
        error
            .attempts()
            .iter()
            .map(|attempt| attempt.provider_id().as_str())
            .collect::<Vec<_>>(),
    );
}

/// Verifies that unknown named selection fails before provider creation.
#[test]
fn test_registry_reports_named_unknown_before_creation() {
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "known",
        &[],
        0,
        ConfigurableProvider::success("known").with_calls(Arc::clone(&calls)),
    );
    let selection = ProviderSelection::named("missing").expect("test selector should be valid");

    let error = registry
        .resolve_selected(&selection)
        .expect_err("unknown selection should fail");
    let message = error.to_string();

    let ProviderResolutionError::UnknownProviders { selectors, .. } = error else {
        panic!("named selection should retain its unknown selector");
    };
    assert_eq!(1, selectors.len());
    assert_eq!("missing", selectors[0].as_str());
    assert_eq!("unknown provider selector; missing", message);
    assert_eq!(0, calls.load(Ordering::SeqCst));
}

/// Verifies that an explicitly lenient chain without matches has no candidates.
#[test]
fn test_registry_reports_chain_without_candidates_before_creation() {
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "known",
        &[],
        0,
        ConfigurableProvider::success("known").with_calls(Arc::clone(&calls)),
    );
    let selection = ProviderSelection::chain_allowing_missing(["first", "second"]).expect("test chain should be valid");

    let error = registry
        .resolve_selected(&selection)
        .expect_err("unmatched chain should fail");
    let message = error.to_string();

    let ProviderResolutionError::NoCandidates { selectors, .. } = error else {
        panic!("unmatched chain should retain all requested selectors");
    };
    assert_eq!(
        vec!["first", "second"],
        selectors.iter().map(|selector| selector.as_str()).collect::<Vec<_>>(),
    );
    assert_eq!("no provider candidates matched; first; second", message,);
    assert_eq!(0, calls.load(Ordering::SeqCst));
}

/// Verifies that strict-chain diagnostics retain every unknown occurrence.
#[test]
fn test_registry_preserves_ordered_duplicate_unknown_selectors() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "known", &[], 0, ConfigurableProvider::success("known"));
    let selection = ProviderSelection::chain(["missing", "known", "missing"]).expect("strict chain should parse");

    let error = registry
        .resolve_selected(&selection)
        .expect_err("strict chain should reject unknown selectors");

    let ProviderResolutionError::UnknownProviders { selectors, .. } = error else {
        panic!("strict chain should report unknown selectors");
    };
    assert_eq!(
        ["missing", "missing"],
        selectors
            .iter()
            .map(|selector| selector.as_str())
            .collect::<Vec<_>>()
            .as_slice(),
    );
}

/// Verifies that automatic selection rejects an empty registry.
#[test]
fn test_registry_reports_empty_auto_selection_before_creation() {
    let registry = ProviderRegistry::<StringSpec>::default();

    let error = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect_err("empty automatic selection should fail");

    assert!(matches!(&error, ProviderResolutionError::EmptyRegistry));
    assert_eq!("cannot select a provider from an empty registry", error.to_string(),);
}

/// Verifies that the never policy stops after the first leaf failure.
#[test]
fn test_never_stops_after_first_failure() {
    let fallback_calls = Arc::new(AtomicUsize::new(0));
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "first",
        &[],
        20,
        ConfigurableProvider::failure(TestProviderFailure::unavailable("offline")),
    );
    register_provider(
        &registry,
        "fallback",
        &[],
        10,
        ConfigurableProvider::success("fallback").with_calls(Arc::clone(&fallback_calls)),
    );
    let selection = ProviderSelection::auto().with_fallback_policy(FallbackPolicy::Never);

    let error = registry
        .resolve_selected(&selection)
        .expect("automatic selection should resolve")
        .create()
        .expect_err("never policy should stop after one failure");

    assert_eq!(ProviderCreationTermination::StoppedByPolicy, error.termination(),);
    assert_eq!(1, error.attempts().len());
    assert_eq!(0, fallback_calls.load(Ordering::SeqCst));
}

/// Verifies absence fallback after unsupported and unavailable providers.
#[test]
fn test_on_absence_continues_after_unsupported_and_unavailable() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "unsupported",
        &[],
        30,
        ConfigurableProvider::failure(TestProviderFailure::unsupported("no format")),
    );
    register_provider(
        &registry,
        "unavailable",
        &[],
        20,
        ConfigurableProvider::failure(TestProviderFailure::unavailable("offline")),
    );
    register_provider(
        &registry,
        "fallback",
        &[],
        10,
        ConfigurableProvider::success("fallback"),
    );

    let output = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve")
        .create()
        .expect("absence policy should reach fallback");

    assert_eq!("fallback", output);
}

/// Verifies absence fallback stops on configuration and initialization errors.
#[test]
fn test_on_absence_stops_after_invalid_config_and_initialization_failure() {
    for error in [
        TestProviderFailure::invalid_configuration("bad config"),
        TestProviderFailure::initialization_failed("broken"),
    ] {
        let registry = ProviderRegistry::<StringSpec>::default();
        register_provider(&registry, "first", &[], 20, ConfigurableProvider::failure(error));
        register_provider(
            &registry,
            "fallback",
            &[],
            10,
            ConfigurableProvider::success("fallback"),
        );

        let error = registry
            .resolve_selected(&ProviderSelection::auto())
            .expect("automatic selection should resolve")
            .create()
            .expect_err("absence policy should stop");

        assert_eq!(ProviderCreationTermination::StoppedByPolicy, error.termination(),);
        assert_eq!(1, error.attempts().len());
    }
}

/// Verifies any-error fallback after every leaf error classification.
#[test]
fn test_on_any_error_continues_after_every_leaf_failure_kind() {
    for error in [
        TestProviderFailure::unsupported("unsupported"),
        TestProviderFailure::unavailable("unavailable"),
        TestProviderFailure::invalid_configuration("invalid"),
        TestProviderFailure::initialization_failed("broken"),
    ] {
        let registry = ProviderRegistry::<StringSpec>::default();
        register_provider(&registry, "first", &[], 20, ConfigurableProvider::failure(error));
        register_provider(
            &registry,
            "fallback",
            &[],
            10,
            ConfigurableProvider::success("fallback"),
        );
        let selection = ProviderSelection::auto().with_fallback_policy(FallbackPolicy::OnAnyError);

        let output = registry
            .resolve_selected(&selection)
            .expect("automatic selection should resolve")
            .create()
            .expect("any-error policy should reach fallback");

        assert_eq!("fallback", output);
    }
}

/// Verifies ordered provider identities and complete source chains on failure.
#[test]
fn test_creation_error_preserves_ordered_provider_ids_and_sources() {
    let registry = ProviderRegistry::<StringSpec>::default();
    for (id, priority) in [("first", 20), ("second", 10)] {
        register_provider(
            &registry,
            id,
            &[],
            priority,
            ConfigurableProvider::failure(TestProviderFailure::unavailable_with_source(
                format!("{id} unavailable"),
                std::io::Error::other(format!("{id} source")),
            )),
        );
    }
    let selection = ProviderSelection::auto().with_fallback_policy(FallbackPolicy::OnAnyError);

    let error = registry
        .resolve_selected(&selection)
        .expect("automatic selection should resolve")
        .create()
        .expect_err("both providers should fail");

    assert_eq!(ProviderCreationTermination::Exhausted, error.termination(),);
    assert_eq!(
        vec!["first", "second"],
        error
            .attempts()
            .iter()
            .map(|attempt| attempt.provider_id().as_str())
            .collect::<Vec<_>>(),
    );
    for attempt in error.attempts() {
        assert!(Error::source(attempt).is_some());
        assert!(Error::source(attempt).and_then(Error::source).is_some());
    }
}

/// Verifies that named creation failures retain the selected canonical ID.
#[test]
fn test_named_selection_failure_contains_exact_provider_id() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "remote",
        &["cloud"],
        0,
        ConfigurableProvider::failure(TestProviderFailure::unavailable("offline")),
    );
    let selection = ProviderSelection::named("cloud").expect("test selector should be valid");

    let error = registry
        .resolve_selected(&selection)
        .expect("named alias should resolve")
        .create()
        .expect_err("selected provider should fail");

    assert_eq!(1, error.attempts().len());
    assert_eq!("remote", error.attempts()[0].provider_id().as_str());
}

/// Verifies that resolved candidates form a stable point-in-time snapshot.
#[test]
fn test_resolved_candidate_snapshot_ignores_later_registration() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "initial", &[], 10, ConfigurableProvider::success("initial"));
    let snapshot = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("initial selection should resolve");

    register_provider(&registry, "later", &[], 20, ConfigurableProvider::success("later"));

    assert_eq!("initial", snapshot.create().expect("snapshot provider should succeed"),);
    assert_eq!(
        "later",
        registry
            .resolve_selected(&ProviderSelection::auto())
            .expect("updated selection should resolve")
            .create()
            .expect("new highest-priority provider should succeed"),
    );
}

/// Verifies concurrent creation through cloned resolving providers.
#[test]
fn test_cloned_resolving_provider_supports_concurrent_creation() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "echo", &[], 0, ConfigurableProvider::echo());
    let provider = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve");

    let threads = (0..8)
        .map(|index| {
            let provider = provider.clone();
            thread::spawn(move || {
                let config = format!("config-{index}");
                provider
                    .create_configured(&config)
                    .expect("echo provider should succeed")
            })
        })
        .collect::<Vec<_>>();

    for (index, thread) in threads.into_iter().enumerate() {
        assert_eq!(
            format!("config-{index}"),
            thread.join().expect("creation thread should not panic"),
        );
    }
}

/// Registers one test provider with descriptor metadata.
///
/// # Parameters
///
/// * `registry` - Runtime registry receiving the provider.
/// * `id` - Canonical provider identity.
/// * `aliases` - Alternative selectors owned by the provider.
/// * `priority` - Descending automatic-selection priority.
/// * `provider` - Service provider fixture or composing provider.
fn register_provider<P>(registry: &ProviderRegistry<StringSpec>, id: &str, aliases: &[&str], priority: i32, provider: P)
where
    P: ServiceProvider<StringSpec>,
{
    let descriptor = ProviderDescriptor::new(ProviderId::new(id).expect("test provider ID should be valid"))
        .with_aliases(aliases.iter().copied())
        .expect("test aliases should be valid")
        .with_priority(priority);
    registry
        .register(define_provider(descriptor, provider))
        .expect("unique test provider should register");
}

/// Provider fixture that panics during configured service creation.
struct PanickingProvider;

impl ServiceProvider<StringSpec> for PanickingProvider {
    /// Panics to verify resolver propagation semantics.
    fn create_configured(&self, _config: &String) -> Result<String, ProviderFailure<TestError>> {
        panic!("test provider panic");
    }
}
