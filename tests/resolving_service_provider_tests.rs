// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::{
    error::Error,
    sync::{
        Arc,
        Mutex,
        atomic::{
            AtomicUsize,
            Ordering,
        },
    },
    thread,
};

use qubit_spi::error::{
    ProviderError,
    ProviderErrorKind,
    ProviderResolutionError,
};
use qubit_spi::{
    FallbackPolicy,
    ProviderCreationTermination,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderSelection,
    ResolvingServiceProvider,
    ServiceProvider,
};

use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_provider_definition::define_provider;

/// Verifies that successful creation returns only the requested service value.
#[test]
fn test_resolving_provider_returns_service_output_directly() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "memory",
        &[],
        0,
        ConfigurableProvider::success("service"),
    );

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
    let seen_config = seen_config
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
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

/// Verifies that a named alias selects exactly its owning provider.
#[test]
fn test_registry_resolves_named_alias_to_one_candidate() {
    let unrelated_calls = Arc::new(AtomicUsize::new(0));
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "english",
        &["en"],
        0,
        ConfigurableProvider::success("hello"),
    );
    register_provider(
        &registry,
        "unrelated",
        &[],
        100,
        ConfigurableProvider::success("other")
            .with_calls(Arc::clone(&unrelated_calls)),
    );
    let selection =
        ProviderSelection::named("EN").expect("test selector should be valid");

    let output = registry
        .resolve_selected(&selection)
        .expect("alias should resolve")
        .create()
        .expect("selected provider should succeed");

    assert_eq!("hello", output);
    assert_eq!(0, unrelated_calls.load(Ordering::SeqCst));
}

/// Verifies chain order, unknown skipping, and alias-based deduplication.
#[test]
fn test_registry_resolves_chain_in_input_order_and_deduplicates_aliases() {
    let first_calls = Arc::new(AtomicUsize::new(0));
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "first",
        &["one"],
        0,
        ConfigurableProvider::failure(ProviderError::unavailable("offline"))
            .with_calls(Arc::clone(&first_calls)),
    );
    register_provider(
        &registry,
        "second",
        &[],
        0,
        ConfigurableProvider::success("second"),
    );
    let selection =
        ProviderSelection::chain(["missing", "one", "first", "second"])
            .expect("test chain should be valid")
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
        register_provider(
            &registry,
            id,
            &[],
            priority,
            ConfigurableProvider::success(id),
        );
    }

    let output = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("automatic selection should resolve")
        .create()
        .expect("highest-ranked provider should succeed");

    assert_eq!("alpha", output);
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
    let selection = ProviderSelection::named("missing")
        .expect("test selector should be valid");

    let error = registry
        .resolve_selected(&selection)
        .expect_err("unknown selection should fail");
    let message = error.to_string();

    let ProviderResolutionError::UnknownProvider { selector, .. } = error
    else {
        panic!("named selection should retain its unknown selector");
    };
    assert_eq!("missing", selector.as_str());
    assert_eq!("unknown provider: missing", message);
    assert_eq!(0, calls.load(Ordering::SeqCst));
}

/// Verifies that a chain without matches fails before provider creation.
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
    let selection = ProviderSelection::chain(["first", "second"])
        .expect("test chain should be valid");

    let error = registry
        .resolve_selected(&selection)
        .expect_err("unmatched chain should fail");
    let message = error.to_string();

    let ProviderResolutionError::NoCandidates { selectors, .. } = error else {
        panic!("unmatched chain should retain all requested selectors");
    };
    assert_eq!(
        vec!["first", "second"],
        selectors
            .iter()
            .map(|selector| selector.as_str())
            .collect::<Vec<_>>(),
    );
    assert_eq!("no provider candidates matched; first; second", message,);
    assert_eq!(0, calls.load(Ordering::SeqCst));
}

/// Verifies that automatic selection rejects an empty registry.
#[test]
fn test_registry_reports_empty_auto_selection_before_creation() {
    let registry = ProviderRegistry::<StringSpec>::default();

    let error = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect_err("empty automatic selection should fail");

    assert!(matches!(&error, ProviderResolutionError::EmptyRegistry));
    assert_eq!(
        "cannot select a provider from an empty registry",
        error.to_string(),
    );
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
        ConfigurableProvider::failure(ProviderError::unavailable("offline")),
    );
    register_provider(
        &registry,
        "fallback",
        &[],
        10,
        ConfigurableProvider::success("fallback")
            .with_calls(Arc::clone(&fallback_calls)),
    );
    let selection =
        ProviderSelection::auto().with_fallback_policy(FallbackPolicy::Never);

    let error = registry
        .resolve_selected(&selection)
        .expect("automatic selection should resolve")
        .create()
        .expect_err("never policy should stop after one failure");

    assert_eq!(
        Some(ProviderCreationTermination::StoppedByPolicy),
        error.termination(),
    );
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
        ConfigurableProvider::failure(ProviderError::unsupported("no format")),
    );
    register_provider(
        &registry,
        "unavailable",
        &[],
        20,
        ConfigurableProvider::failure(ProviderError::unavailable("offline")),
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
        ProviderError::invalid_configuration("bad config"),
        ProviderError::initialization_failed("broken"),
    ] {
        let registry = ProviderRegistry::<StringSpec>::default();
        register_provider(
            &registry,
            "first",
            &[],
            20,
            ConfigurableProvider::failure(error),
        );
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

        assert_eq!(
            Some(ProviderCreationTermination::StoppedByPolicy),
            error.termination(),
        );
        assert_eq!(1, error.attempts().len());
    }
}

/// Verifies any-error fallback after every leaf error classification.
#[test]
fn test_on_any_error_continues_after_every_leaf_failure_kind() {
    for error in [
        ProviderError::unsupported("unsupported"),
        ProviderError::unavailable("unavailable"),
        ProviderError::invalid_configuration("invalid"),
        ProviderError::initialization_failed("broken"),
    ] {
        let registry = ProviderRegistry::<StringSpec>::default();
        register_provider(
            &registry,
            "first",
            &[],
            20,
            ConfigurableProvider::failure(error),
        );
        register_provider(
            &registry,
            "fallback",
            &[],
            10,
            ConfigurableProvider::success("fallback"),
        );
        let selection = ProviderSelection::auto()
            .with_fallback_policy(FallbackPolicy::OnAnyError);

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
            ConfigurableProvider::failure(
                ProviderError::unavailable_with_source(
                    format!("{id} unavailable"),
                    std::io::Error::other(format!("{id} source")),
                ),
            ),
        );
    }
    let selection = ProviderSelection::auto()
        .with_fallback_policy(FallbackPolicy::OnAnyError);

    let error = registry
        .resolve_selected(&selection)
        .expect("automatic selection should resolve")
        .create()
        .expect_err("both providers should fail");

    assert_eq!(
        Some(ProviderCreationTermination::Exhausted),
        error.termination(),
    );
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
        ConfigurableProvider::failure(ProviderError::unavailable("offline")),
    );
    let selection = ProviderSelection::named("cloud")
        .expect("test selector should be valid");

    let error = registry
        .resolve_selected(&selection)
        .expect("named alias should resolve")
        .create()
        .expect_err("selected provider should fail");

    assert_eq!(1, error.attempts().len());
    assert_eq!("remote", error.attempts()[0].provider_id().as_str());
}

/// Verifies that a nested aggregate becomes a terminal initialization failure.
#[test]
fn test_nested_aggregate_candidate_error_stops_as_initialization_failure() {
    let inner_registry = ProviderRegistry::<StringSpec>::default();
    for (id, priority) in [("inner-first", 20), ("inner-second", 10)] {
        register_provider(
            &inner_registry,
            id,
            &[],
            priority,
            ConfigurableProvider::failure(ProviderError::unavailable(
                "offline",
            )),
        );
    }
    let inner_selection = ProviderSelection::auto()
        .with_fallback_policy(FallbackPolicy::OnAnyError);
    let inner_provider = inner_registry
        .resolve_selected(&inner_selection)
        .expect("inner selection should resolve");

    let fallback_calls = Arc::new(AtomicUsize::new(0));
    let outer_registry = ProviderRegistry::<StringSpec>::default();
    register_provider(&outer_registry, "composed", &[], 20, inner_provider);
    register_provider(
        &outer_registry,
        "fallback",
        &[],
        10,
        ConfigurableProvider::success("fallback")
            .with_calls(Arc::clone(&fallback_calls)),
    );
    let outer_selection = ProviderSelection::auto()
        .with_fallback_policy(FallbackPolicy::OnAnyError);

    let error = outer_registry
        .resolve_selected(&outer_selection)
        .expect("outer selection should resolve")
        .create()
        .expect_err("nested aggregate must stop outer fallback");

    assert_eq!(1, error.attempts().len());
    assert_eq!(
        ProviderErrorKind::InitializationFailed,
        error.attempts()[0].error().kind(),
    );
    assert!(Error::source(error.attempts()[0].error()).is_some());
    assert_eq!(0, fallback_calls.load(Ordering::SeqCst));
}

/// Verifies that resolved candidates form a stable point-in-time snapshot.
#[test]
fn test_resolved_candidate_snapshot_ignores_later_registration() {
    let registry = ProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "initial",
        &[],
        10,
        ConfigurableProvider::success("initial"),
    );
    let snapshot = registry
        .resolve_selected(&ProviderSelection::auto())
        .expect("initial selection should resolve");

    register_provider(
        &registry,
        "later",
        &[],
        20,
        ConfigurableProvider::success("later"),
    );

    assert_eq!(
        "initial",
        snapshot.create().expect("snapshot provider should succeed"),
    );
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
/// # Arguments
///
/// * `registry` - Runtime registry receiving the provider.
/// * `id` - Canonical provider identity.
/// * `aliases` - Alternative selectors owned by the provider.
/// * `priority` - Descending automatic-selection priority.
/// * `provider` - Service provider fixture or composing provider.
fn register_provider<P>(
    registry: &ProviderRegistry<StringSpec>,
    id: &str,
    aliases: &[&str],
    priority: i32,
    provider: P,
) where
    P: ServiceProvider<StringSpec>,
{
    let descriptor = ProviderDescriptor::new(
        ProviderId::new(id).expect("test provider ID should be valid"),
    )
    .with_aliases(aliases.iter().copied())
    .expect("test aliases should be valid")
    .with_priority(priority);
    registry
        .register(define_provider(descriptor, provider))
        .expect("unique test provider should register");
}
