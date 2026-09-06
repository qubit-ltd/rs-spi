// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::fmt::Write;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use futures::executor::block_on;
use qubit_spi::AsyncProviderDefinition;
use qubit_spi::AsyncProviderRegistry;
use qubit_spi::AsyncServiceProvider;
use qubit_spi::FallbackPolicy;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::ProviderSelection;
use qubit_spi::error::ProviderResolutionError;

use crate::common::async_configurable_provider::AsyncConfigurableProvider;
use crate::common::blocking_writer::BlockingWriter;
use crate::common::string_spec::StringSpec;
use crate::common::test_error::TestProviderFailure;
use crate::common::test_provider_definition::define_provider;

/// Verifies synchronous registration and resolution with asynchronous creation.
#[test]
fn test_async_registry_registers_and_resolves_without_await() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "async", &[], 0, AsyncConfigurableProvider::echo());

    let resolver = registry
        .resolve_selected(&ProviderSelection::named("async").expect("test selection should parse"))
        .expect("selection should resolve synchronously");

    assert_eq!(
        "config",
        block_on(resolver.create_configured(&"config".to_owned())).expect("async creation should succeed"),
    );
    assert_eq!(
        ["async"],
        registry
            .provider_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
            .as_slice()
    );
}

/// Verifies strict missing-selector semantics in the asynchronous facade.
#[test]
fn test_async_registry_uses_strict_chain_semantics() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "known", &[], 0, AsyncConfigurableProvider::success("known"));

    let error = registry
        .resolve_selected(&ProviderSelection::chain(["missing", "known"]).expect("strict chain should parse"))
        .expect_err("strict chain should fail before returning a future");

    assert!(matches!(
        error,
        ProviderResolutionError::UnknownProviders { selectors, .. }
            if selectors.len() == 1 && selectors[0].as_str() == "missing"
    ));
}

/// Verifies explicit missing-selector tolerance and candidate deduplication.
#[test]
fn test_async_registry_allows_explicit_missing_chain_entries() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "first",
        &["one"],
        0,
        AsyncConfigurableProvider::failure(TestProviderFailure::unavailable("offline")),
    );
    register_provider(
        &registry,
        "second",
        &[],
        0,
        AsyncConfigurableProvider::success("second"),
    );
    let selection = ProviderSelection::chain_allowing_missing(["missing", "one", "first", "second"])
        .expect("lenient chain should parse")
        .with_fallback_policy(FallbackPolicy::OnAnyError);

    let resolver = registry
        .resolve_selected(&selection)
        .expect("known candidates should resolve");
    assert_eq!("second", block_on(resolver.create()).expect("fallback should succeed"),);
}

/// Verifies all synchronous catalog facade operations and shared registration.
#[test]
fn test_async_registry_exposes_synchronous_catalog_snapshots() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    assert!(registry.is_empty());
    assert_eq!(ProviderSelection::auto(), registry.default_selection());

    let descriptor = ProviderDescriptor::new(ProviderId::new("shared").expect("static ID should be valid"));
    let provider: Arc<dyn AsyncProviderDefinition<StringSpec>> = Arc::new(define_provider(
        descriptor.clone(),
        AsyncConfigurableProvider::success("shared"),
    ));
    registry
        .register_shared(provider)
        .expect("shared provider should register");
    let clone = registry.clone();
    let selection = ProviderSelection::named("shared").expect("static selection should be valid");
    clone.set_default_selection(selection.clone());

    assert!(!registry.is_empty());
    assert_eq!(1, registry.len());
    assert_eq!([descriptor], registry.descriptors().as_slice());
    assert_eq!(selection, registry.default_selection());
    assert_eq!(
        "shared",
        block_on(registry.resolve().expect("default selection should resolve").create(),)
            .expect("shared provider should create"),
    );
    assert!(format!("{registry:?}").contains("shared"));
}

/// Verifies a successful asynchronous default snapshot keeps its candidates after registration.
#[test]
fn test_async_registry_default_snapshot_keeps_successful_resolution() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "first", &[], 0, AsyncConfigurableProvider::success("first"));
    registry.set_default_selection(ProviderSelection::auto());

    let (selection, snapshot) = registry
        .resolve_default_snapshot_with_selection()
        .expect("default snapshot should resolve");
    assert_eq!(ProviderSelection::auto(), selection);

    register_provider(&registry, "second", &[], 100, AsyncConfigurableProvider::success("second"));

    assert_eq!(
        "first",
        block_on(snapshot.create()).expect("snapshot should retain first provider"),
    );
    assert_eq!(
        "second",
        block_on(
            registry
                .resolve_default_snapshot()
                .expect("new default snapshot should resolve")
                .create(),
        )
        .expect("new snapshot should select second provider"),
    );
}

/// Verifies a failed asynchronous default snapshot remains an owned result after registration.
#[test]
fn test_async_registry_default_snapshot_keeps_failed_resolution() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    registry.set_default_selection(ProviderSelection::named("missing").expect("test selection should be valid"));

    let error = registry
        .resolve_default_snapshot()
        .expect_err("missing default provider should fail");

    register_provider(&registry, "missing", &[], 0, AsyncConfigurableProvider::success("now-present"));

    assert!(matches!(
        error,
        ProviderResolutionError::UnknownProviders { selectors, .. }
            if selectors.len() == 1 && selectors[0].as_str() == "missing"
    ));
}

/// Verifies asynchronous Registry Debug retains one metadata snapshot.
#[test]
fn test_async_registry_debug_uses_one_metadata_snapshot() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    registry.set_default_selection(ProviderSelection::named("before").expect("static selection should be valid"));
    let formatting_registry = registry.clone();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let formatter = thread::spawn(move || {
        let mut writer = BlockingWriter::new("descriptors", entered_tx, release_rx);
        write!(&mut writer, "{formatting_registry:?}").expect("coordinated formatting should succeed");
        writer.into_output()
    });

    entered_rx.recv().expect("formatter should reach the descriptors field");
    registry.set_default_selection(ProviderSelection::named("after").expect("static selection should be valid"));
    release_tx
        .send(())
        .expect("formatter should remain blocked until released");
    let debug = formatter.join().expect("formatter thread should not panic");

    assert!(debug.contains("before"), "unexpected Debug output: {debug}");
    assert!(!debug.contains("after"), "mixed Debug snapshot: {debug}");
}

/// Registers one metadata-bearing asynchronous test provider.
pub(crate) fn register_provider<P>(
    registry: &AsyncProviderRegistry<StringSpec>,
    id: &str,
    aliases: &[&str],
    priority: i32,
    provider: P,
) where
    P: AsyncServiceProvider<StringSpec>,
{
    let descriptor = ProviderDescriptor::new(ProviderId::new(id).expect("test provider ID should be valid"))
        .with_aliases(aliases.iter().copied())
        .expect("test aliases should be valid")
        .with_priority(priority);
    registry
        .register(define_provider(descriptor, provider))
        .expect("unique async provider should register");
}
