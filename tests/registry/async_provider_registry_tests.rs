#![allow(unused_must_use)]
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
use qubit_spi::error::RegistryMutationError;

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

#[test]
fn test_async_registry_seal_is_shared_by_clones() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    let clone = registry.clone();
    registry.seal();
    assert!(registry.is_sealed());
    assert!(clone.is_sealed());
    assert!(
        clone
            .set_default_selection(ProviderSelection::auto())
            .expect_err("sealed registry rejects selection mutation")
            .is_sealed()
    );
}

/// Verifies a successful asynchronous default snapshot keeps its candidates
/// after registration.
#[test]
fn test_async_registry_default_snapshot_keeps_successful_resolution() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "first", &[], 0, AsyncConfigurableProvider::success("first"));
    registry.set_default_selection(ProviderSelection::auto());

    let (selection, snapshot) = registry.resolve_default_snapshot();
    let snapshot = snapshot.expect("default snapshot should resolve");
    assert_eq!(ProviderSelection::auto(), selection);

    register_provider(
        &registry,
        "second",
        &[],
        100,
        AsyncConfigurableProvider::success("second"),
    );

    assert_eq!(
        "first",
        block_on(snapshot.create()).expect("snapshot should retain first provider"),
    );
    assert_eq!(
        "second",
        block_on(
            registry
                .resolve_default_snapshot()
                .1
                .expect("new default snapshot should resolve")
                .create(),
        )
        .expect("new snapshot should select second provider"),
    );
}

/// Verifies a failed asynchronous default snapshot remains an owned result
/// after registration.
#[test]
fn test_async_registry_default_snapshot_keeps_failed_resolution() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    registry.set_default_selection(ProviderSelection::named("missing").expect("test selection should be valid"));

    let (selection, result) = registry.resolve_default_snapshot();
    assert_eq!(ProviderSelection::named("missing").unwrap(), selection);
    let error = result.expect_err("missing default provider should fail");

    register_provider(
        &registry,
        "missing",
        &[],
        0,
        AsyncConfigurableProvider::success("now-present"),
    );

    assert!(matches!(
        error,
        ProviderResolutionError::UnknownProviders { selectors, .. }
            if selectors.len() == 1 && selectors[0].as_str() == "missing"
    ));
}

/// Verifies asynchronous Registry Debug retains one metadata snapshot.
#[test]
fn test_async_registry_debug_uses_one_metadata_snapshot() {
    if !crate::common::subprocess_case::enter() {
        return;
    }
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

/// Metadata is sampled once; provider mutations do not change registered
/// selectors.
#[test]
fn test_descriptor_is_sampled_once_and_owned_by_registry() {
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use crate::common::registry_contract_provider::RegistryContractProvider;
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    let calls = Arc::new(AtomicUsize::new(0));
    let changed = Arc::new(AtomicBool::new(false));
    let observed_calls = Arc::clone(&calls);
    let observed_changed = Arc::clone(&changed);
    registry
        .register(RegistryContractProvider {
            metadata: Box::new(move || {
                observed_calls.fetch_add(1, Ordering::SeqCst);
                let id = if observed_changed.load(Ordering::SeqCst) {
                    "later"
                } else {
                    "original"
                };
                ProviderDescriptor::new(ProviderId::new(id).expect("valid fixture ID"))
                    .with_aliases(["initial-alias"])
                    .expect("valid alias")
                    .with_priority(i32::MAX)
            }),
            on_drop: None,
        })
        .expect("provider registers");
    changed.store(true, Ordering::SeqCst);
    let mut descriptors = registry.descriptors();
    assert_eq!("original", descriptors[0].id().as_str());
    assert_eq!(i32::MAX, descriptors[0].priority());
    descriptors.clear();
    assert_eq!(1, registry.descriptors().len());
    assert!(
        registry
            .resolve_selected(&ProviderSelection::named("later").expect("valid selector"))
            .is_err()
    );
    assert!(
        registry
            .resolve_selected(&ProviderSelection::named("initial-alias").expect("valid selector"))
            .is_ok()
    );
    assert_eq!(1, calls.load(Ordering::SeqCst));
}

/// Metadata and rejected-provider destruction must run outside catalog locks.
#[test]
fn test_descriptor_and_rejected_drop_can_reenter_registry() {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use crate::common::registry_contract_provider::RegistryContractProvider;
    if !crate::common::subprocess_case::enter() {
        return;
    }
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    let reentrant = registry.clone();
    registry
        .register(RegistryContractProvider {
            metadata: Box::new(move || {
                assert!(reentrant.is_empty());
                reentrant.set_default_selection(ProviderSelection::auto());
                ProviderDescriptor::new(ProviderId::new("original").expect("valid ID"))
            }),
            on_drop: None,
        })
        .expect("reentrant metadata registers");
    let before_ids = registry.provider_ids();
    let before_descriptors = registry.descriptors();
    let drops = Arc::new(AtomicUsize::new(0));
    let observed_drops = Arc::clone(&drops);
    let reentrant = registry.clone();
    let result = registry.register(RegistryContractProvider {
        metadata: Box::new(|| ProviderDescriptor::new(ProviderId::new("original").expect("valid ID"))),
        on_drop: Some(Box::new(move || {
            reentrant.set_default_selection(ProviderSelection::auto());
            assert_eq!(1, reentrant.descriptors().len());
            observed_drops.fetch_add(1, Ordering::SeqCst);
        })),
    });
    assert!(matches!(result, Err(RegistryMutationError::DuplicateSelector { .. })));
    assert_eq!(1, drops.load(Ordering::SeqCst));
    assert_eq!(before_ids, registry.provider_ids());
    assert_eq!(before_descriptors, registry.descriptors());
}

/// A metadata panic leaves the catalog intact and usable for later
/// registration.
#[test]
fn test_descriptor_panic_does_not_mutate_or_poison_registry() {
    use std::panic::AssertUnwindSafe;
    use std::panic::catch_unwind;

    use crate::common::registry_contract_provider::RegistryContractProvider;
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    let before_ids = registry.provider_ids();
    let before_descriptors = registry.descriptors();
    let result = catch_unwind(AssertUnwindSafe(|| {
        registry.register(RegistryContractProvider {
            metadata: Box::new(|| panic!("metadata panic")),
            on_drop: None,
        })
    }));
    assert!(result.is_err());
    assert_eq!(before_ids, registry.provider_ids());
    assert_eq!(before_descriptors, registry.descriptors());
    registry
        .register(RegistryContractProvider {
            metadata: Box::new(|| ProviderDescriptor::new(ProviderId::new("after-panic").expect("valid ID"))),
            on_drop: None,
        })
        .expect("registry remains writable");
}

/// Each returned selection governs its own resolver despite concurrent default
/// changes.
#[test]
fn test_default_snapshot_is_consistent_during_concurrent_updates() {
    if !crate::common::subprocess_case::enter() {
        return;
    }
    use std::sync::Barrier;

    use qubit_spi::ProviderCreationTermination;

    use crate::common::test_error::TestProviderFailure;
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("a").expect("valid ID")).with_priority(i32::MIN),
            AsyncConfigurableProvider::success("a"),
        ))
        .expect("a registers");
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("b").expect("valid ID")).with_priority(i32::MAX),
            AsyncConfigurableProvider::failure(TestProviderFailure::unavailable("b absent")),
        ))
        .expect("b registers");
    let selections = [
        ProviderSelection::named("a").expect("valid selection"),
        ProviderSelection::auto().with_fallback_policy(FallbackPolicy::Never),
        ProviderSelection::chain(["b", "a"])
            .expect("valid chain")
            .with_fallback_policy(FallbackPolicy::OnAnyError),
        ProviderSelection::named("b")
            .expect("valid selection")
            .with_fallback_policy(FallbackPolicy::Never),
    ];
    registry.set_default_selection(selections[0].clone());
    let barrier = Barrier::new(2);
    thread::scope(|scope| {
        scope.spawn(|| {
            for iteration in 0..512 {
                barrier.wait();
                registry.set_default_selection(selections[iteration % selections.len()].clone());
                barrier.wait();
            }
        });
        for _ in 0..512 {
            barrier.wait();
            let (selection, resolver) = registry.resolve_default_snapshot();
            barrier.wait();
            let resolver = resolver.expect("all known defaults resolve");
            let result = block_on(resolver.create());
            if selection == selections[0] || selection == selections[2] {
                assert_eq!("a", result.expect("selected path reaches a"));
            } else {
                assert!(selection == selections[1] || selection == selections[3]);
                let error = result.expect_err("selected path stops at b");
                assert_eq!(1, error.attempts().len());
                assert_eq!("b", error.attempts()[0].provider_id().as_str());
                assert_eq!(
                    if selection == selections[1] {
                        ProviderCreationTermination::StoppedByPolicy
                    } else {
                        ProviderCreationTermination::Exhausted
                    },
                    error.termination(),
                );
            }
        }
    });
}

/// Registration reentered from metadata wins before the outer conflict check.
#[test]
fn test_reentrant_metadata_registration_is_visible_to_outer_validation() {
    use crate::common::registry_contract_provider::RegistryContractProvider;
    if !crate::common::subprocess_case::enter() {
        return;
    }
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    let reentrant = registry.clone();
    let result = registry.register(RegistryContractProvider {
        metadata: Box::new(move || {
            reentrant
                .register(RegistryContractProvider {
                    metadata: Box::new(|| ProviderDescriptor::new(ProviderId::new("nested").expect("valid ID"))),
                    on_drop: None,
                })
                .expect("nested provider registers without an outer write lock");
            ProviderDescriptor::new(ProviderId::new("outer").expect("valid ID"))
                .with_aliases(["nested"])
                .expect("valid alias")
        }),
        on_drop: None,
    });
    assert!(matches!(result, Err(RegistryMutationError::DuplicateSelector { .. })));
    assert_eq!(
        vec![ProviderId::new("nested").expect("valid ID")],
        registry.provider_ids()
    );
    assert_eq!("nested", registry.descriptors()[0].id().as_str());
    assert!(
        registry
            .resolve_selected(&ProviderSelection::named("outer").expect("valid selector"))
            .is_err()
    );
}

/// A panicking outer descriptor does not roll back a completed reentrant
/// registration.
#[test]
fn test_descriptor_panic_preserves_reentrant_registration() {
    use std::panic::AssertUnwindSafe;
    use std::panic::catch_unwind;

    use crate::common::registry_contract_provider::RegistryContractProvider;
    if !crate::common::subprocess_case::enter() {
        return;
    }
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    let reentrant = registry.clone();
    let result = catch_unwind(AssertUnwindSafe(|| {
        registry.register(RegistryContractProvider {
            metadata: Box::new(move || {
                reentrant
                    .register(RegistryContractProvider {
                        metadata: Box::new(|| ProviderDescriptor::new(ProviderId::new("nested").expect("valid ID"))),
                        on_drop: None,
                    })
                    .expect("nested registration completes before the panic");
                panic!("outer metadata panic");
            }),
            on_drop: None,
        })
    }));
    assert!(result.is_err());
    assert_eq!(
        vec![ProviderId::new("nested").expect("valid ID")],
        registry.provider_ids()
    );
    assert_eq!(
        vec![ProviderDescriptor::new(ProviderId::new("nested").expect("valid ID"))],
        registry.descriptors()
    );
    registry
        .resolve_selected(&ProviderSelection::named("nested").expect("valid selector"))
        .expect("completed reentrant registration remains resolvable");
}
