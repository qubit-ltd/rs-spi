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

use qubit_spi::FallbackPolicy;
use qubit_spi::ProviderDefinition;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::ProviderRegistry;
use qubit_spi::ProviderSelection;
use qubit_spi::error::ProviderResolutionError;
use qubit_spi::error::RegistryMutationError;

use crate::common::blocking_writer::BlockingWriter;
use crate::common::configurable_provider::ConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_provider_definition::define_provider;

/// Verifies that a registry accepts a self-described provider after creation.
#[test]
fn test_registry_registers_a_self_described_provider_at_runtime() {
    let registry = ProviderRegistry::<StringSpec>::default();

    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("english").expect("test provider ID should be valid")),
            ConfigurableProvider::success("hello"),
        ))
        .expect("runtime registration should succeed");

    assert_eq!(1, registry.len());
    assert_eq!("english", registry.provider_ids()[0].as_str());
}

/// Verifies registration of an already shared synchronous provider.
#[test]
fn test_registry_registers_an_existing_shared_provider() {
    let registry = ProviderRegistry::<StringSpec>::default();
    let provider: Arc<dyn ProviderDefinition<StringSpec>> = Arc::new(define_provider(
        ProviderDescriptor::new(ProviderId::new("shared").expect("test provider ID should be valid")),
        ConfigurableProvider::success("shared"),
    ));

    registry
        .register_shared(provider)
        .expect("shared provider should register");

    assert_eq!("shared", registry.provider_ids()[0].as_str());
}

/// Verifies that a failed registration leaves every selector unclaimed.
#[test]
fn test_registry_rejects_conflicts_without_partial_mutation() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("english").expect("test provider ID should be valid"))
                .with_aliases(["en"])
                .expect("test alias should be valid"),
            ConfigurableProvider::success("hello"),
        ))
        .expect("first provider should register");

    let error = registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("spanish").expect("test provider ID should be valid"))
                .with_aliases(["es", "en"])
                .expect("test aliases should be valid"),
            ConfigurableProvider::success("hola"),
        ))
        .expect_err("duplicate alias should be rejected");

    assert!(matches!(error, RegistryMutationError::DuplicateSelector { .. }));
    assert_eq!(
        vec!["english"],
        registry
            .provider_ids()
            .iter()
            .map(ProviderId::as_str)
            .collect::<Vec<_>>()
    );
    let selection = ProviderSelection::named("es").expect("test selector should be valid");
    assert!(matches!(
        registry.resolve_selected(&selection),
        Err(ProviderResolutionError::UnknownProviders { .. }),
    ));
}

/// Verifies that a duplicate canonical ID leaves the registry unchanged.
#[test]
fn test_registry_rejects_duplicate_canonical_id_without_partial_mutation() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("english").expect("test provider ID should be valid")),
            ConfigurableProvider::success("hello"),
        ))
        .expect("first provider should register");

    let error = registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("english").expect("test provider ID should be valid")),
            ConfigurableProvider::success("bonjour"),
        ))
        .expect_err("duplicate canonical ID should be rejected");

    assert!(matches!(error, RegistryMutationError::DuplicateSelector { .. }));
    assert_eq!(
        vec!["english"],
        registry
            .provider_ids()
            .iter()
            .map(ProviderId::as_str)
            .collect::<Vec<_>>(),
    );
}

/// Verifies that cloned handles observe providers registered later.
#[test]
fn test_registry_clones_share_later_registrations() {
    let registry = ProviderRegistry::<StringSpec>::default();
    let clone = registry.clone();

    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("english").expect("test provider ID should be valid")),
            ConfigurableProvider::success("hello"),
        ))
        .expect("runtime registration should succeed");

    assert_eq!(1, clone.len());
    assert_eq!(
        vec!["english"],
        clone.provider_ids().iter().map(ProviderId::as_str).collect::<Vec<_>>()
    );
}

#[test]
fn test_registry_seal_is_idempotent_and_shared_by_clones() {
    let registry = ProviderRegistry::<StringSpec>::default();
    let clone = registry.clone();
    assert!(!registry.is_sealed());
    registry.seal();
    registry.seal();
    assert!(registry.is_sealed());
    assert!(clone.is_sealed());
    let error = registry
        .set_default_selection(ProviderSelection::auto())
        .expect_err("sealed registry rejects selection mutation");
    assert!(error.is_sealed());
}

/// Verifies reading and replacing the registry's default selection.
#[test]
fn test_registry_uses_and_updates_default_selection() {
    let registry = ProviderRegistry::<StringSpec>::default();
    assert_eq!(ProviderSelection::auto(), registry.default_selection());

    let selection = ProviderSelection::named("english")
        .expect("test selector should be valid")
        .with_fallback_policy(FallbackPolicy::OnAnyError);
    registry.set_default_selection(selection.clone());

    assert_eq!(selection, registry.default_selection());
}

/// Verifies default resolution and registry debug metadata snapshots.
#[test]
fn test_registry_resolves_configured_default_and_formats_snapshot() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("english").expect("test provider ID should be valid")),
            ConfigurableProvider::success("hello"),
        ))
        .expect("test provider should register");
    registry.set_default_selection(ProviderSelection::named("english").expect("test selector should be valid"));

    let output = registry
        .resolve()
        .expect("default selection should resolve")
        .create()
        .expect("default provider should create its service");
    let debug = format!("{registry:?}");

    assert_eq!("hello", output);
    assert!(debug.contains("ProviderRegistry"));
    assert!(debug.contains("english"));
    assert!(debug.contains("default_selection"));
}

/// Verifies a successful default snapshot keeps its candidates after
/// registration.
#[test]
fn test_registry_default_snapshot_keeps_successful_resolution() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("first").expect("test provider ID should be valid")),
            ConfigurableProvider::success("first"),
        ))
        .expect("first provider should register");
    registry.set_default_selection(ProviderSelection::auto());

    let (selection, snapshot) = registry.resolve_default_snapshot();
    let snapshot = snapshot.expect("default snapshot should resolve");
    assert_eq!(ProviderSelection::auto(), selection);

    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("second").expect("test provider ID should be valid"))
                .with_priority(100),
            ConfigurableProvider::success("second"),
        ))
        .expect("second provider should register");

    assert_eq!(
        "first",
        snapshot.create().expect("snapshot should retain first provider")
    );
    let (_, snapshot) = registry.resolve_default_snapshot();
    assert_eq!(
        "second",
        snapshot
            .expect("new default snapshot should resolve")
            .create()
            .expect("new snapshot should select second provider"),
    );
}

/// Verifies a failed default snapshot remains an owned result after
/// registration.
#[test]
fn test_registry_default_snapshot_keeps_failed_resolution() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry.set_default_selection(ProviderSelection::named("missing").expect("test selection should be valid"));

    let (selection, result) = registry.resolve_default_snapshot();
    assert_eq!(ProviderSelection::named("missing").unwrap(), selection);
    let error = result.expect_err("missing default provider should fail");

    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("missing").expect("test provider ID should be valid")),
            ConfigurableProvider::success("now-present"),
        ))
        .expect("provider should register after failed snapshot");

    assert!(matches!(
        error,
        ProviderResolutionError::UnknownProviders { selectors, .. }
            if selectors.len() == 1 && selectors[0].as_str() == "missing"
    ));
}

/// Verifies Registry Debug formatting retains one metadata snapshot.
#[test]
fn test_registry_debug_uses_one_metadata_snapshot() {
    if !crate::common::subprocess_case::enter() {
        return;
    }
    let registry = ProviderRegistry::<StringSpec>::default();
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

/// Verifies that descriptor snapshots retain successful registration order.
#[test]
fn test_registry_preserves_registration_order_in_descriptor_snapshots() {
    let registry = ProviderRegistry::<StringSpec>::default();
    for id in ["third", "first", "second"] {
        registry
            .register(define_provider(
                ProviderDescriptor::new(ProviderId::new(id).expect("test provider ID should be valid")),
                ConfigurableProvider::success("hello"),
            ))
            .expect("unique provider should register");
    }

    assert_eq!(
        vec!["third", "first", "second"],
        registry
            .descriptors()
            .iter()
            .map(|descriptor| descriptor.id().as_str())
            .collect::<Vec<_>>(),
    );
}

/// Verifies concurrent registration and owned metadata snapshots.
#[test]
fn test_registry_supports_concurrent_registration_and_snapshot_reads() {
    let registry = ProviderRegistry::<StringSpec>::default();
    let threads = (0..8)
        .map(|index| {
            let registry = registry.clone();
            thread::spawn(move || {
                let id = format!("provider-{index}");
                registry
                    .register(define_provider(
                        ProviderDescriptor::new(ProviderId::new(&id).expect("test provider ID should be valid")),
                        ConfigurableProvider::success("hello"),
                    ))
                    .expect("unique provider should register");
                registry.descriptors()
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        assert!(!thread.join().expect("registration thread should not panic").is_empty());
    }
    let mut ids = registry
        .provider_ids()
        .into_iter()
        .map(|id| id.as_str().to_owned())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    assert_eq!((0..8).map(|index| format!("provider-{index}")).collect::<Vec<_>>(), ids,);
}

/// Verifies registry size and emptiness before and after registration.
#[test]
fn test_registry_length_matches_emptiness_and_registration_count() {
    let empty = ProviderRegistry::<StringSpec>::default();
    assert_eq!(0, empty.len());
    assert!(empty.is_empty());
    assert!(empty.provider_ids().is_empty());
    assert!(empty.descriptors().is_empty());

    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("english").expect("valid ID")),
            ConfigurableProvider::success("hello"),
        ))
        .expect("unique provider should register");
    assert_eq!(1, registry.len());
    assert!(!registry.is_empty());
}

/// Metadata is sampled once; provider mutations do not change registered
/// selectors.
#[test]
fn test_descriptor_is_sampled_once_and_owned_by_registry() {
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use crate::common::registry_contract_provider::RegistryContractProvider;
    let registry = ProviderRegistry::<StringSpec>::default();
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
    let registry = ProviderRegistry::<StringSpec>::default();
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
    let registry = ProviderRegistry::<StringSpec>::default();
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
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("a").expect("valid ID")).with_priority(i32::MIN),
            ConfigurableProvider::success("a"),
        ))
        .expect("a registers");
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("b").expect("valid ID")).with_priority(i32::MAX),
            ConfigurableProvider::failure(TestProviderFailure::unavailable("b absent")),
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
            let result = resolver.create();
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

/// Both canonical-to-alias and alias-to-canonical conflicts leave all metadata
/// intact.
#[test]
fn test_registry_rejects_cross_kind_selector_collisions_atomically() {
    let registry = ProviderRegistry::<StringSpec>::default();
    registry
        .register(define_provider(
            ProviderDescriptor::new(ProviderId::new("original").expect("valid ID"))
                .with_aliases(["owned-alias"])
                .expect("valid alias"),
            ConfigurableProvider::success("original"),
        ))
        .expect("initial provider registers");
    let before = registry.descriptors();
    for (id, aliases) in [("owned-alias", vec!["fresh"]), ("fresh", vec!["original"])] {
        let result = registry.register(define_provider(
            ProviderDescriptor::new(ProviderId::new(id).expect("valid ID"))
                .with_aliases(aliases)
                .expect("valid alias"),
            ConfigurableProvider::success("rejected"),
        ));
        assert!(matches!(result, Err(RegistryMutationError::DuplicateSelector { .. })));
        assert_eq!(before, registry.descriptors());
        assert!(
            registry
                .resolve_selected(&ProviderSelection::named("fresh").expect("valid selector"))
                .is_err()
        );
    }
}

/// Registration reentered from metadata wins before the outer conflict check.
#[test]
fn test_reentrant_metadata_registration_is_visible_to_outer_validation() {
    use crate::common::registry_contract_provider::RegistryContractProvider;
    if !crate::common::subprocess_case::enter() {
        return;
    }
    let registry = ProviderRegistry::<StringSpec>::default();
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
    let registry = ProviderRegistry::<StringSpec>::default();
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
