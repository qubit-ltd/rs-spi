mod support;

use std::error::Error;
use std::io;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Once;

use qubit_spi::{
    ProviderCreateError,
    ProviderName,
    ProviderRegistryError,
    ProviderSelection,
    ServiceProvider,
};

use crate::support::test_services::{
    GreetingProvider,
    GreetingRegistry,
    GreetingSpec,
    TestConfig,
};

/// No-op logger used to exercise diagnostic logging paths.
struct TestLogger;

impl log::Log for TestLogger {
    /// Tells the log facade that all records are accepted by this test logger.
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true
    }

    /// Drops one log record.
    fn log(&self, _record: &log::Record<'_>) {}

    /// Flushes the no-op logger.
    fn flush(&self) {}
}

/// Shared no-op logger instance.
static TEST_LOGGER: TestLogger = TestLogger;

/// Ensures the global test logger is installed only once.
static TEST_LOGGER_INIT: Once = Once::new();

/// Installs a trace-level no-op logger for diagnostic-path coverage.
fn init_test_logger() {
    TEST_LOGGER_INIT.call_once(|| {
        let _ignored = log::set_logger(&TEST_LOGGER);
        log::set_max_level(log::LevelFilter::Trace);
    });
}

/// Exercises registry paths that emit diagnostic log records.
fn exercise_registry_diagnostics() {
    let empty_registry = GreetingRegistry::new();
    assert!(matches!(
        empty_registry
            .create_auto_box(&TestConfig::new(""))
            .expect_err("empty registry should fail"),
        ProviderRegistryError::EmptyRegistry
    ));

    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("low", "low").with_priority(1))
        .expect("low provider should register");
    registry
        .register(GreetingProvider::new("unavailable", "hello").unavailable("disabled"))
        .expect("unavailable provider should register");
    registry
        .register(GreetingProvider::new("failing", "hello").failing("boom"))
        .expect("failing provider should register");

    assert!(registry.resolve_provider("low").is_ok());
    assert!(registry.resolve_provider("missing").is_err());
    assert!(registry.resolve_provider("bad provider").is_err());
    assert!(registry.create_box("low", &TestConfig::new("")).is_ok());
    assert!(registry.create_box("unavailable", &TestConfig::new("")).is_err());
    assert!(registry.create_box("failing", &TestConfig::new("")).is_err());
    assert!(registry.create_auto_box(&TestConfig::new("")).is_ok());

    let failing_selection =
        ProviderSelection::from_names("missing", &["unavailable", "failing"]).expect("selection names should be valid");
    assert!(
        registry
            .create_selected_box(&failing_selection, &TestConfig::new(""))
            .is_err()
    );

    let fallback_selection =
        ProviderSelection::from_names("unavailable", &["low"]).expect("selection names should be valid");
    assert!(
        registry
            .create_selected_box(&fallback_selection, &TestConfig::new(""))
            .is_ok()
    );
}

/// Test new registries start empty.
#[test]
fn test_new_registry_is_empty() {
    let registry = GreetingRegistry::new();

    assert!(registry.is_empty());
    assert_eq!(0, registry.len());
    assert!(registry.provider_names().is_empty());
}

/// Test diagnostic logging paths do not change registry behavior.
#[test]
fn test_diagnostic_logging_paths_do_not_change_registry_behavior() {
    exercise_registry_diagnostics();
    init_test_logger();
    exercise_registry_diagnostics();
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
        .create_box("STATIC", &config)
        .expect("provider id should resolve");
    let by_alias = registry
        .create_box(" static-greeter ", &config)
        .expect("provider alias should resolve");
    let descriptors = registry.provider_descriptors();
    let iter_names: Vec<_> = registry.iter_provider_names().collect();
    let iter_descriptor_ids: Vec<_> = registry
        .iter_provider_descriptors()
        .map(|descriptor| descriptor.id().as_str())
        .collect();

    assert_eq!(vec!["static"], registry.provider_names());
    assert_eq!(vec!["static"], iter_names);
    assert_eq!(vec!["static"], iter_descriptor_ids);
    assert_eq!("static", descriptors[0].id().as_str());
    assert_eq!(vec!["static-greeter"], descriptors[0].aliases_as_str());
    assert!(registry.find_provider("static-greeter").is_some());
    assert_eq!("say hello", by_id.greet());
    assert_eq!("say hello", by_alias.greet());
}

/// Test resolving providers reports invalid and unknown names precisely.
#[test]
fn test_resolve_provider_reports_name_errors() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("static", "hello").with_aliases(&["static-greeter"]))
        .expect("provider should register");

    let provider = registry
        .resolve_provider(" STATIC-GREETER ")
        .expect("provider alias should resolve");
    let invalid_error = registry
        .resolve_provider("bad provider")
        .expect_err("invalid provider names should fail");
    let missing_error = registry
        .resolve_provider("missing")
        .expect_err("unknown providers should fail");

    assert_eq!(
        "static",
        provider
            .descriptor()
            .expect("descriptor should remain valid")
            .id()
            .as_str(),
    );
    assert!(matches!(
        invalid_error,
        ProviderRegistryError::InvalidProviderName { ref name, .. }
            if name == "bad provider"
    ));
    assert!(matches!(
        missing_error,
        ProviderRegistryError::UnknownProvider { ref name } if name.as_str() == "missing"
    ));
}

/// Test shared provider registration keeps the same provider instance.
#[test]
fn test_register_shared_uses_shared_provider_instance() {
    let provider = Arc::new(GreetingProvider::new("shared", "hello"));
    let mut registry = GreetingRegistry::new();

    registry
        .register_shared(provider.clone())
        .expect("shared provider should register");
    let service = registry
        .create_box("shared", &TestConfig::new(""))
        .expect("shared provider should create a service");

    assert_eq!("hello", service.greet());
    assert_eq!(1, provider.created_count());
}

/// Test shared registration accepts an already-erased provider trait object.
#[test]
fn test_register_shared_accepts_erased_provider_instance() {
    let provider: Arc<dyn ServiceProvider<GreetingSpec>> = Arc::new(GreetingProvider::new("erased", "hello"));
    let mut registry = GreetingRegistry::new();

    registry
        .register_shared(provider)
        .expect("erased provider should register");
    let service = registry
        .create_box("erased", &TestConfig::new(""))
        .expect("erased provider should create a service");

    assert_eq!("hello", service.greet());
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
        ProviderRegistryError::DuplicateProviderName { ref name } if name.as_str() == "fast"
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

/// Test invalid provider ids are rejected.
#[test]
fn test_register_rejects_invalid_provider_name() {
    let mut registry = GreetingRegistry::new();
    let error = registry
        .register(GreetingProvider::new("native provider", "hello"))
        .expect_err("invalid provider id should be rejected");

    assert!(matches!(
        error,
        ProviderRegistryError::InvalidProviderName { ref name, .. } if name == "native provider"
    ));
}

/// Test unknown selectors fail with an unknown-provider error.
#[test]
fn test_create_reports_unknown_provider() {
    let registry = GreetingRegistry::new();
    let error = registry
        .create_box("missing", &TestConfig::new(""))
        .expect_err("unknown provider should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::UnknownProvider { ref name } if name.as_str() == "missing"
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
        .create_box("native", &TestConfig::new(""))
        .expect_err("unavailable provider should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::ProviderUnavailable { ref name, ref source }
            if name.as_str() == "native" && source.reason() == "not installed"
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
        .create_box("native", &TestConfig::new(""))
        .expect_err("failing provider should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::ProviderCreate { ref name, ref source }
            if name.as_str() == "native" && source.reason() == "boom"
    ));
}

/// Test provider creation errors preserve nested sources through the registry.
#[test]
fn test_create_preserves_provider_creation_source() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(
            GreetingProvider::new("native", "hello").failing_with(ProviderCreateError::failed_with_source(
                "boom",
                io::Error::other("root cause"),
            )),
        )
        .expect("provider should register");

    let error = registry
        .create_box("native", &TestConfig::new(""))
        .expect_err("failing provider should fail");
    let ProviderRegistryError::ProviderCreate { source, .. } = error else {
        panic!("expected ProviderCreate");
    };

    assert_eq!("boom", source.reason());
    assert_eq!(
        "root cause",
        Error::source(&source).expect("source should be preserved").to_string(),
    );
}

/// Test provider-reported unavailability is mapped to registry context.
#[test]
fn test_create_maps_provider_unavailable_creation_error() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("native", "hello").failing_unavailable("runtime missing"))
        .expect("provider should register");

    let error = registry
        .create_box("native", &TestConfig::new(""))
        .expect_err("provider should report unavailable during creation");

    assert!(matches!(
        error,
        ProviderRegistryError::ProviderUnavailable { ref name, ref source }
            if name.as_str() == "native" && source.reason() == "runtime missing"
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
        .create_auto_box(&TestConfig::new(""))
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
        .create_auto_box(&TestConfig::new(""))
        .expect("auto selection should create first provider by id");

    assert_eq!("a", service.greet());
}

/// Test Arc and Rc creation variants preserve registry selection behavior.
#[test]
fn test_create_shared_variants_preserve_selection_behavior() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("low", "low").with_priority(1))
        .expect("low provider should register");
    registry
        .register(GreetingProvider::new("high", "high").with_priority(10))
        .expect("high provider should register");
    let selection = ProviderSelection::from_names("missing", &["low"]).expect("selection names should be valid");

    let auto_arc: Arc<_> = registry
        .create_auto_arc(&TestConfig::new(""))
        .expect("auto arc selection should create highest priority provider");
    let auto_rc: Rc<_> = registry
        .create_auto_rc(&TestConfig::new(""))
        .expect("auto rc selection should create highest priority provider");
    let selected_arc: Arc<_> = registry
        .create_selected_arc(&selection, &TestConfig::new(""))
        .expect("selected arc fallback should create fallback provider");
    let selected_rc: Rc<_> = registry
        .create_selected_rc(&selection, &TestConfig::new(""))
        .expect("selected rc fallback should create fallback provider");

    assert_eq!("high", auto_arc.greet());
    assert_eq!("high", auto_rc.greet());
    assert_eq!("low", selected_arc.greet());
    assert_eq!("low", selected_rc.greet());
}

/// Test explicit fallback chains continue after unavailable and failing providers.
#[test]
fn test_create_selected_uses_fallback_chain() {
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
    let selection =
        ProviderSelection::from_names("primary", &["secondary", "fallback"]).expect("selection names should be valid");

    let service = registry
        .create_selected_box(&selection, &TestConfig::new(""))
        .expect("fallback provider should be used");

    assert_eq!("fallback", service.greet());
}

/// Test manually duplicated selection candidates are rejected before execution.
#[test]
fn test_create_selected_rejects_duplicate_candidate_names() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("native", "hello"))
        .expect("native provider should register");
    let selection = ProviderSelection::Named {
        primary: ProviderName::new("native").expect("primary name should be valid"),
        fallbacks: vec![ProviderName::new("NATIVE").expect("fallback name should be valid")],
    };

    let error = registry
        .create_selected_box(&selection, &TestConfig::new(""))
        .expect_err("duplicate selection candidates should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::DuplicateProviderCandidate { ref name } if name.as_str() == "native"
    ));
}

/// Test aliases that resolve to an already-tried provider are not retried.
#[test]
fn test_create_selected_skips_aliases_for_already_tried_provider() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(
            GreetingProvider::new("native", "hello")
                .with_aliases(&["fast"])
                .unavailable("not installed"),
        )
        .expect("native provider should register");
    let selection =
        ProviderSelection::from_names("native", &["fast"]).expect("distinct candidate names should be accepted");

    let error = registry
        .create_selected_box(&selection, &TestConfig::new(""))
        .expect_err("all candidates should fail");

    let ProviderRegistryError::NoAvailableProvider { failures } = error else {
        panic!("expected NoAvailableProvider");
    };
    assert_eq!(1, failures.len());
    assert_eq!(
        "provider 'native' is unavailable: not installed",
        failures[0].to_string(),
    );
}

/// Test failed candidate chains report all failures.
#[test]
fn test_create_selected_reports_all_candidate_failures() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("native", "hello").unavailable("not installed"))
        .expect("native provider should register");
    registry
        .register(GreetingProvider::new("fallback", "hello").failing("boom"))
        .expect("fallback provider should register");
    let selection =
        ProviderSelection::from_names("missing", &["native", "fallback"]).expect("selection names should be valid");

    let error = registry
        .create_selected_box(&selection, &TestConfig::new(""))
        .expect_err("all failed candidates should be reported");

    let ProviderRegistryError::NoAvailableProvider { failures } = error else {
        panic!("expected NoAvailableProvider");
    };
    assert_eq!(3, failures.len());
    assert_eq!("unknown provider: missing", failures[0].to_string());
    assert_eq!(
        "provider 'native' is unavailable: not installed",
        failures[1].to_string(),
    );
    assert_eq!(
        "provider 'fallback' failed to create service: boom",
        failures[2].to_string(),
    );
}

/// Test fallback aggregation maps provider creation failures to candidate failures.
#[test]
fn test_create_selected_converts_create_error_to_candidate_failure() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(
            GreetingProvider::new("primary", "hello").failing_with(ProviderCreateError::failed("backend exploded")),
        )
        .expect("primary provider should register");
    let selection = ProviderSelection::from_names("primary", &[]).expect("selection names should be valid");

    let error = registry
        .create_selected_box(&selection, &TestConfig::new(""))
        .expect_err("creation errors should be reported as candidate failures");

    let ProviderRegistryError::NoAvailableProvider { failures } = error else {
        panic!("expected NoAvailableProvider");
    };
    assert_eq!(1, failures.len());
    assert_eq!(
        "provider 'primary' failed to create service: backend exploded",
        failures[0].to_string(),
    );
}

/// Test fallback aggregation preserves provider-reported unavailable creation errors.
#[test]
fn test_create_selected_converts_provider_unavailable_create_error() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("primary", "hello").failing_unavailable("runtime missing"))
        .expect("primary provider should register");
    registry
        .register(GreetingProvider::new("fallback", "hello").failing("boom"))
        .expect("fallback provider should register");
    let selection = ProviderSelection::from_names("primary", &["fallback"]).expect("selection names should be valid");

    let error = registry
        .create_selected_box(&selection, &TestConfig::new(""))
        .expect_err("provider creation errors should be aggregated");

    let ProviderRegistryError::NoAvailableProvider { failures } = error else {
        panic!("expected NoAvailableProvider");
    };
    assert_eq!(2, failures.len());
    assert_eq!(
        "provider 'primary' is unavailable: runtime missing",
        failures[0].to_string(),
    );
    assert_eq!(
        "provider 'fallback' failed to create service: boom",
        failures[1].to_string(),
    );
}

/// Test auto selection reports an empty registry distinctly.
#[test]
fn test_create_auto_reports_empty_registry() {
    let registry = GreetingRegistry::new();
    let error = registry
        .create_auto_box(&TestConfig::new(""))
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

/// Test cloned registries keep the automatic selection order snapshot.
#[test]
fn test_clone_preserves_auto_selection_snapshot() {
    let mut registry = GreetingRegistry::new();
    registry
        .register(GreetingProvider::new("first", "first"))
        .expect("first provider should register");
    let snapshot = registry.clone();

    registry
        .register(GreetingProvider::new("second", "second").with_priority(100))
        .expect("second provider should register");

    let snapshot_service = snapshot
        .create_auto_box(&TestConfig::new(""))
        .expect("snapshot should use first provider");
    let registry_service = registry
        .create_auto_box(&TestConfig::new(""))
        .expect("registry should use higher-priority provider");

    assert_eq!("first", snapshot_service.greet());
    assert_eq!("second", registry_service.greet());
}
