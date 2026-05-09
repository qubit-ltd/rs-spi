mod support;

use std::sync::Arc;

use qubit_spi::{
    ProviderFailure,
    ProviderRegistryError,
    ProviderSelection,
    ServiceProvider,
};

use crate::support::test_services::{
    GreetingProvider,
    GreetingRegistry,
    GreetingService,
    TestConfig,
    TestProviderError,
};

/// Test new registries start empty.
#[test]
fn test_new_registry_is_empty() {
    let registry = GreetingRegistry::new();

    assert!(registry.is_empty());
    assert_eq!(0, registry.len());
    assert!(registry.provider_names().is_empty());
}

/// Test registration exposes providers by id and alias case-insensitively.
#[test]
fn test_register_finds_and_creates_by_id_and_alias_case_insensitively() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("static", "hello").with_aliases(&["static-greeter"]))
        .expect("provider should register");

    let config = TestConfig::new("say ");
    let by_id = registry
        .create("STATIC", &config)
        .expect("provider id should resolve");
    let by_alias = registry
        .create(" static-greeter ", &config)
        .expect("provider alias should resolve");

    assert_eq!(vec!["static"], registry.provider_names());
    assert!(registry.find_provider("static-greeter").is_some());
    assert_eq!("say hello", by_id.greet());
    assert_eq!("say hello", by_alias.greet());
}

/// Test shared provider registration keeps the same provider instance.
#[test]
fn test_register_arc_uses_shared_provider_instance() {
    let provider = Arc::new(GreetingProvider::new("shared", "hello"));
    let provider_for_registry: Arc<
        dyn ServiceProvider<
                Config = TestConfig,
                Error = TestProviderError,
                Service = dyn GreetingService,
            >,
    > = provider.clone();
    let mut registry = GreetingRegistry::new();

    registry
        .register_arc(provider_for_registry)
        .expect("shared provider should register");
    let service = registry
        .create("shared", &TestConfig::new(""))
        .expect("shared provider should create a service");

    assert_eq!("hello", service.greet());
    assert_eq!(1, provider.created_count());
}

/// Test duplicate ids and aliases are rejected case-insensitively.
#[test]
fn test_register_rejects_duplicate_provider_names() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("native", "hello").with_aliases(&["fast"]))
        .expect("first provider should register");

    let error = registry
        .register(GreetingProvider::new("other", "hello").with_aliases(&["FAST"]))
        .expect_err("duplicate alias should be rejected");

    assert!(matches!(
        error,
        ProviderRegistryError::DuplicateProviderName { ref name } if name == "FAST"
    ));
}

/// Test empty provider ids are rejected.
#[test]
fn test_register_rejects_empty_provider_name() {
    let mut registry = GreetingRegistry::new();
    let error = registry
        .register(GreetingProvider::new(" ", "hello"))
        .expect_err("empty provider id should be rejected");

    assert!(matches!(error, ProviderRegistryError::EmptyProviderName));
}

/// Test unknown selectors fail with an unknown-provider error.
#[test]
fn test_create_reports_unknown_provider() {
    let registry = GreetingRegistry::new();
    let error = registry
        .create("missing", &TestConfig::new(""))
        .expect_err("unknown provider should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::UnknownProvider { ref name } if name == "missing"
    ));
}

/// Test unavailable providers fail before creation.
#[test]
fn test_create_reports_unavailable_provider() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("native", "hello").unavailable("not installed"))
        .expect("provider should register");

    let error = registry
        .create("native", &TestConfig::new(""))
        .expect_err("unavailable provider should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::ProviderUnavailable { ref name, ref reason }
            if name == "native" && reason == "not installed"
    ));
}

/// Test provider creation errors are preserved.
#[test]
fn test_create_wraps_provider_creation_error() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("native", "hello").failing("boom"))
        .expect("provider should register");

    let error = registry
        .create("native", &TestConfig::new(""))
        .expect_err("failing provider should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::ProviderCreate { ref name, ref error }
            if name == "native" && error == &TestProviderError::new("boom")
    ));
}

/// Test automatic selection prefers the highest-priority provider.
#[test]
fn test_create_auto_selects_highest_priority_provider() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("low", "low").with_priority(1))
        .expect("low provider should register");
    registry
        .register(GreetingProvider::new("high", "high").with_priority(10))
        .expect("high provider should register");

    let service = registry
        .create_auto(&TestConfig::new(""))
        .expect("auto selection should create highest priority provider");

    assert_eq!("high", service.greet());
}

/// Test automatic selection uses provider id as a deterministic tiebreaker.
#[test]
fn test_create_auto_tie_breaks_by_provider_id() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("z-provider", "z").with_priority(10))
        .expect("z provider should register");
    registry
        .register(GreetingProvider::new("a-provider", "a").with_priority(10))
        .expect("a provider should register");

    let service = registry
        .create_auto(&TestConfig::new(""))
        .expect("auto selection should create first provider by id");

    assert_eq!("a", service.greet());
}

/// Test explicit fallback chains continue after unavailable and failing providers.
#[test]
fn test_create_default_uses_fallback_chain() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("primary", "primary").unavailable("disabled"))
        .expect("primary provider should register");
    registry
        .register(GreetingProvider::new("secondary", "secondary").failing("boom"))
        .expect("secondary provider should register");
    registry
        .register(GreetingProvider::new("fallback", "fallback"))
        .expect("fallback provider should register");
    let selection = ProviderSelection::from_names("primary", &["secondary", "fallback"]);

    let service = registry
        .create_default(&selection, &TestConfig::new(""))
        .expect("fallback provider should be used");

    assert_eq!("fallback", service.greet());
}

/// Test failed candidate chains report all failures.
#[test]
fn test_create_default_reports_all_candidate_failures() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("native", "hello").unavailable("not installed"))
        .expect("native provider should register");
    registry
        .register(GreetingProvider::new("fallback", "hello").failing("boom"))
        .expect("fallback provider should register");
    let selection = ProviderSelection::from_names("missing", &["native", "fallback"]);

    let error = registry
        .create_default(&selection, &TestConfig::new(""))
        .expect_err("all failed candidates should be reported");

    let ProviderRegistryError::NoAvailableProvider { failures } = error else {
        panic!("expected NoAvailableProvider");
    };
    assert_eq!(
        vec![
            ProviderFailure::<TestProviderError>::unknown("missing"),
            ProviderFailure::unavailable("native", "not installed"),
            ProviderFailure::create_failed("fallback", TestProviderError::new("boom")),
        ],
        failures,
    );
}

/// Test auto selection reports an empty registry distinctly.
#[test]
fn test_create_auto_reports_empty_registry() {
    let registry = GreetingRegistry::new();
    let error = registry
        .create_auto(&TestConfig::new(""))
        .expect_err("empty registry should fail");

    assert!(matches!(error, ProviderRegistryError::EmptyRegistry));
}

/// Test cloned registries are independent provider-list snapshots.
#[test]
fn test_clone_creates_independent_provider_list_snapshot() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("first", "first"))
        .expect("first provider should register");
    let snapshot = registry.clone();

    registry
        .register(GreetingProvider::new("second", "second"))
        .expect("second provider should register");

    assert!(snapshot.find_provider("first").is_some());
    assert!(snapshot.find_provider("second").is_none());
    assert!(registry.find_provider("second").is_some());
}
