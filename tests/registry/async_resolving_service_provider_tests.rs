// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
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
use std::sync::mpsc;
use std::thread;

use futures::channel::oneshot;
use futures::executor::block_on;
use futures::task::noop_waker;
use qubit_spi::AsyncProviderRegistry;
use qubit_spi::AsyncServiceProvider;
use qubit_spi::FallbackPolicy;
use qubit_spi::ProviderCreationTermination;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderFuture;
use qubit_spi::ProviderId;
use qubit_spi::ProviderMetadata;
use qubit_spi::ProviderSelection;
use qubit_spi::error::ProviderFailure;
use qubit_spi::error::ProviderFailureKind;

use crate::common::async_configurable_provider::AsyncConfigurableProvider;
use crate::common::string_spec::StringSpec;
use crate::common::test_error::TestError;
use crate::common::test_error::TestProviderFailure;
use crate::registry::async_provider_registry_tests::register_provider;

/// Requires a value to implement [`Send`].
fn assert_send<T: Send>(_: T) {}

/// Verifies resolver creation futures can cross executor threads.
#[test]
fn test_async_resolver_creation_futures_are_send() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "sendable",
        &[],
        10,
        AsyncConfigurableProvider::success("value"),
    );
    let resolver = registry.resolve().expect("provider should resolve");
    let config = String::new();

    assert_send(resolver.create_configured(&config));
    assert_send(resolver.create());
}

/// Verifies fallback policy behavior after awaited leaf failures.
#[test]
fn test_async_resolver_applies_all_fallback_policies() {
    for (policy, error, succeeds) in [
        (
            FallbackPolicy::Never,
            TestProviderFailure::unavailable("offline"),
            false,
        ),
        (
            FallbackPolicy::OnAbsence,
            TestProviderFailure::unavailable("offline"),
            true,
        ),
        (
            FallbackPolicy::OnAbsence,
            TestProviderFailure::invalid_configuration("invalid"),
            false,
        ),
        (
            FallbackPolicy::OnAnyError,
            TestProviderFailure::initialization_failed("broken"),
            true,
        ),
    ] {
        let registry = AsyncProviderRegistry::<StringSpec>::default();
        register_provider(&registry, "first", &[], 20, AsyncConfigurableProvider::failure(error));
        register_provider(
            &registry,
            "fallback",
            &[],
            10,
            AsyncConfigurableProvider::success("fallback"),
        );
        let resolver = registry
            .resolve_selected(&ProviderSelection::auto().with_fallback_policy(policy))
            .expect("automatic selection should resolve");
        let result = block_on(resolver.create());

        if succeeds {
            assert_eq!("fallback", result.expect("fallback should succeed"));
        } else {
            assert_eq!(
                ProviderCreationTermination::StoppedByPolicy,
                result.expect_err("fallback should stop").termination(),
            );
        }
    }
}

/// Verifies provider panics propagate without awaiting fallback candidates.
#[test]
fn test_async_resolver_propagates_provider_panic_without_trying_fallback() {
    let fallback_calls = Arc::new(AtomicUsize::new(0));
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    register_provider(&registry, "panicking", &[], 20, AsyncPanickingProvider);
    register_provider(
        &registry,
        "fallback",
        &[],
        10,
        AsyncCountingProvider {
            calls: Arc::clone(&fallback_calls),
        },
    );
    let resolver = registry
        .resolve_selected(&ProviderSelection::auto().with_fallback_policy(FallbackPolicy::OnAnyError))
        .expect("automatic selection should resolve");

    let result = catch_unwind(AssertUnwindSafe(|| block_on(resolver.create())));

    assert!(result.is_err(), "provider panic should propagate");
    assert_eq!(0, fallback_calls.load(Ordering::SeqCst));
}

/// Verifies ordered attempt diagnostics and original sources.
#[test]
fn test_async_resolver_retains_attempt_order_and_sources() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    for (id, priority) in [("first", 20), ("second", 10)] {
        register_provider(
            &registry,
            id,
            &[],
            priority,
            AsyncConfigurableProvider::failure(TestProviderFailure::unavailable_with_source(
                format!("{id} unavailable"),
                std::io::Error::other(format!("{id} source")),
            )),
        );
    }
    let resolver = registry
        .resolve_selected(&ProviderSelection::auto().with_fallback_policy(FallbackPolicy::OnAnyError))
        .expect("automatic selection should resolve");

    let error = block_on(resolver.create()).expect_err("every provider should fail");

    assert_eq!(ProviderCreationTermination::Exhausted, error.termination());
    assert_eq!(
        ["first", "second"],
        error
            .attempts()
            .iter()
            .map(|attempt| attempt.provider_id().as_str())
            .collect::<Vec<_>>()
            .as_slice(),
    );
    assert!(Error::source(&error).is_some());
}

/// Verifies that resolved asynchronous candidates form a stable snapshot.
#[test]
fn test_async_resolver_snapshot_ignores_later_registration() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "initial",
        &[],
        10,
        AsyncConfigurableProvider::success("initial"),
    );
    let snapshot = registry.resolve().expect("initial provider should resolve");
    register_provider(&registry, "later", &[], 20, AsyncConfigurableProvider::success("later"));

    assert_eq!("initial", block_on(snapshot.create()).expect("snapshot should succeed"));
    assert_eq!(
        "later",
        block_on(registry.resolve().expect("updated selection should resolve").create())
            .expect("updated resolver should succeed"),
    );
}

/// Verifies cloned asynchronous resolvers retain debuggable snapshots.
#[test]
fn test_async_resolver_clone_and_debug_preserve_candidates() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "debuggable",
        &[],
        10,
        AsyncConfigurableProvider::success("value"),
    );
    let resolver = registry.resolve().expect("provider should resolve");
    let clone = resolver.clone();

    assert!(format!("{clone:?}").contains("debuggable"));
    assert_eq!(
        "value",
        block_on(clone.create()).expect("cloned resolver should create"),
    );
}

/// Provider that announces entry and remains pending until explicitly released.
struct PendingProvider {
    descriptor: ProviderDescriptor,
    entered: Mutex<Option<mpsc::Sender<()>>>,
    release: Mutex<Option<oneshot::Receiver<()>>>,
}

impl ProviderMetadata for PendingProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }
}

impl AsyncServiceProvider<StringSpec> for PendingProvider {
    fn create_configured<'a>(
        &'a self,
        _config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<TestError>>> {
        let entered = self
            .entered
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
            .expect("pending provider should be called once");
        let release = self
            .release
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
            .expect("pending provider should be called once");
        Box::pin(async move {
            entered.send(()).expect("entry receiver should remain alive");
            release.await.map_err(|error| {
                TestProviderFailure::initialization_failed_with_source("release signal was canceled", error)
            })?;
            Ok("released".to_owned())
        })
    }
}

/// Verifies that awaiting provider creation holds no Registry lock.
#[test]
fn test_async_provider_pending_does_not_hold_registry_lock() {
    if !crate::common::subprocess_case::enter() {
        return;
    }
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = oneshot::channel();
    registry
        .register(PendingProvider {
            descriptor: ProviderDescriptor::new(ProviderId::new("pending").expect("test provider ID should be valid")),
            entered: Mutex::new(Some(entered_tx)),
            release: Mutex::new(Some(release_rx)),
        })
        .expect("pending provider should register");
    let resolver = registry.resolve().expect("pending provider should resolve");
    let creation = thread::spawn(move || block_on(resolver.create()));

    entered_rx.recv().expect("provider should announce entry");
    register_provider(&registry, "later", &[], 0, AsyncConfigurableProvider::success("later"));
    assert_eq!(2, registry.provider_ids().len());
    release_tx.send(()).expect("pending provider should remain alive");

    assert_eq!(
        "released",
        creation
            .join()
            .expect("creation thread should not panic")
            .expect("pending provider should succeed"),
    );
}

/// Verifies that a final candidate failure is exhaustion, not a policy stop.
#[test]
fn test_async_final_failure_is_exhausted() {
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    register_provider(
        &registry,
        "only",
        &[],
        0,
        AsyncConfigurableProvider::failure(TestProviderFailure::invalid_configuration("invalid")),
    );

    let error =
        block_on(registry.resolve().expect("provider should resolve").create()).expect_err("only provider should fail");
    assert_eq!(ProviderCreationTermination::Exhausted, error.termination());
    assert_eq!(
        ProviderFailureKind::InvalidConfiguration,
        error.decisive_attempt().failure().kind()
    );
}

/// Provider fixture that panics while its creation future is polled.
struct AsyncPanickingProvider;

impl AsyncServiceProvider<StringSpec> for AsyncPanickingProvider {
    /// Returns a future that panics when the resolver polls it.
    fn create_configured<'a>(
        &'a self,
        _config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<TestError>>> {
        Box::pin(async { panic!("test provider panic") })
    }
}

/// Provider fixture that records calls before returning a stable output.
struct AsyncCountingProvider {
    calls: Arc<AtomicUsize>,
}

impl AsyncServiceProvider<StringSpec> for AsyncCountingProvider {
    /// Records one invocation and returns a stable service output.
    fn create_configured<'a>(
        &'a self,
        _config: &'a String,
    ) -> ProviderFuture<'a, Result<String, ProviderFailure<TestError>>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok("fallback".to_owned()) })
    }
}

/// Dropping at any suspension boundary releases the active future without
/// fallback.
#[test]
fn test_async_resolver_cancellation_never_invokes_later_candidates() {
    use std::future::Future;
    use std::task::Context;
    use std::task::Poll;

    use crate::common::async_drop_provider::AsyncAction;
    use crate::common::async_drop_provider::AsyncDropProvider;
    for (polled, first_absent) in [(false, false), (true, false), (true, true)] {
        let registry = AsyncProviderRegistry::<StringSpec>::default();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let drops = Arc::new(AtomicUsize::new(0));
        if first_absent {
            register_provider(
                &registry,
                "absent",
                &[],
                30,
                AsyncDropProvider {
                    id: "absent",
                    action: AsyncAction::Absent,
                    calls: Arc::clone(&calls),
                    drops: Arc::clone(&drops),
                },
            );
        }
        register_provider(
            &registry,
            "pending",
            &[],
            20,
            AsyncDropProvider {
                id: "pending",
                action: AsyncAction::Pending,
                calls: Arc::clone(&calls),
                drops: Arc::clone(&drops),
            },
        );
        register_provider(
            &registry,
            "later",
            &[],
            10,
            AsyncDropProvider {
                id: "later",
                action: AsyncAction::Success,
                calls: Arc::clone(&calls),
                drops: Arc::clone(&drops),
            },
        );
        let resolver = registry
            .resolve_selected(&ProviderSelection::auto().with_fallback_policy(FallbackPolicy::OnAnyError))
            .expect("registered candidates resolve");
        let config = String::new();
        let mut future = Box::pin(resolver.create_configured(&config));
        if polled {
            let waker = noop_waker();
            let mut context = Context::from_waker(&waker);
            assert!(matches!(future.as_mut().poll(&mut context), Poll::Pending));
        }
        drop(future);
        let expected = if !polled {
            vec![]
        } else if first_absent {
            vec!["absent", "pending"]
        } else {
            vec!["pending"]
        };
        assert_eq!(expected, *calls.lock().expect("event lock must be available"));
        assert_eq!(usize::from(polled), drops.load(Ordering::SeqCst));
    }
}

/// A panic before the leaf returns a future propagates without invoking
/// fallback.
#[test]
fn test_async_resolver_propagates_panic_before_future_construction() {
    use crate::common::async_drop_provider::AsyncAction;
    use crate::common::async_drop_provider::AsyncDropProvider;
    let registry = AsyncProviderRegistry::<StringSpec>::default();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let drops = Arc::new(AtomicUsize::new(0));
    for (id, priority, action) in [("panic", 20, AsyncAction::Panic), ("later", 10, AsyncAction::Success)] {
        register_provider(
            &registry,
            id,
            &[],
            priority,
            AsyncDropProvider {
                id,
                action,
                calls: Arc::clone(&calls),
                drops: Arc::clone(&drops),
            },
        );
    }
    let resolver = registry
        .resolve_selected(&ProviderSelection::auto().with_fallback_policy(FallbackPolicy::OnAnyError))
        .expect("registered candidates resolve");
    assert!(catch_unwind(AssertUnwindSafe(|| block_on(resolver.create()))).is_err());
    assert_eq!(vec!["panic"], *calls.lock().expect("event lock must be available"));
    assert_eq!(0, drops.load(Ordering::SeqCst));
}

/// Every create invokes the factory, but provider-owned resources may be
/// shared.
#[test]
fn test_resolver_clone_preserves_provider_identity_without_caching_outputs() {
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use crate::common::registry_contract_provider::SharedOutputProvider;
    use crate::common::registry_contract_provider::SharedOutputSpec;
    let registry = AsyncProviderRegistry::<SharedOutputSpec>::default();
    let output = Arc::new("shared allocation".to_owned());
    let calls = Arc::new(AtomicUsize::new(0));
    registry
        .register(SharedOutputProvider {
            output: Arc::clone(&output),
            calls: Arc::clone(&calls),
        })
        .expect("shared provider registers");
    let resolver = registry.resolve().expect("provider resolves");
    let cloned = resolver.clone();
    let first = block_on(resolver.create()).expect("infallible provider succeeds");
    let second = block_on(cloned.create()).expect("infallible provider succeeds");
    assert_eq!(2, calls.load(Ordering::SeqCst));
    assert!(Arc::ptr_eq(&output, &first));
    assert!(Arc::ptr_eq(&first, &second));
}
