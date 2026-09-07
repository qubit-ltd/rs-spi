#![allow(unused_must_use)]
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Creation benchmarks include traversal, diagnostics and async executor
//! polling.

use std::future::poll_fn;
use std::hint::black_box;
use std::io::Error;
use std::sync::Barrier;
use std::task::Poll;
use std::thread;
use std::time::Instant;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use futures::executor::block_on;
use qubit_spi::AsyncProviderRegistry;
use qubit_spi::AsyncServiceProvider;
use qubit_spi::AsyncServiceSpec;
use qubit_spi::FallbackPolicy;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderFuture;
use qubit_spi::ProviderId;
use qubit_spi::ProviderMetadata;
use qubit_spi::ProviderRegistry;
use qubit_spi::ProviderSelection;
use qubit_spi::ServiceProvider;
use qubit_spi::ServiceSpec;
use qubit_spi::SyncServiceSpec;
use qubit_spi::error::ProviderFailure;

/// Service fixture excludes external I/O and business resource allocation.
struct CreationSpec;

impl ServiceSpec for CreationSpec {
    type Config = ();
    type Error = Error;
}

impl SyncServiceSpec for CreationSpec {
    type Output = usize;
}

impl AsyncServiceSpec for CreationSpec {
    type Output = usize;
}

/// Provider with a configured result and optional one-poll suspension.
struct CreationProvider {
    index: usize,
    fails: bool,
    pending: bool,
}

impl ProviderMetadata for CreationProvider {
    /// Gives providers a deterministic ascending numeric traversal order.
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(ProviderId::new(&format!("provider-{:03}", self.index)).expect("valid ID"))
    }
}

impl ServiceProvider<CreationSpec> for CreationProvider {
    /// Produces the selected fixture result without external I/O.
    fn create_configured(&self, _config: &()) -> Result<usize, ProviderFailure<Error>> {
        if self.fails {
            Err(ProviderFailure::unavailable(Error::other("fixture unavailable")))
        } else {
            Ok(self.index)
        }
    }
}

impl AsyncServiceProvider<CreationSpec> for CreationProvider {
    /// Optionally suspends once and wakes the executor before returning a
    /// result.
    fn create_configured<'a>(&'a self, config: &'a ()) -> ProviderFuture<'a, Result<usize, ProviderFailure<Error>>> {
        Box::pin(async move {
            let mut pending = self.pending;
            poll_fn(|context| {
                if pending {
                    pending = false;
                    context.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            })
            .await;
            ServiceProvider::create_configured(self, config)
        })
    }
}

/// Measures success, successful fallback and complete exhaustion independently.
fn benchmark_creation(criterion: &mut Criterion) {
    for workload in ["success", "fallback", "exhausted"] {
        let mut group = criterion.benchmark_group(format!("creation_{workload}"));
        for count in [1, 8, 64] {
            let registry = ProviderRegistry::<CreationSpec>::default();
            for index in 0..count {
                registry
                    .register(CreationProvider {
                        index,
                        fails: workload == "exhausted" || (workload == "fallback" && index + 1 < count),
                        pending: false,
                    })
                    .expect("unique provider registers");
            }
            let resolver = registry.resolve().expect("populated registry resolves");
            group.bench_function(BenchmarkId::from_parameter(count), |bencher| {
                bencher.iter(|| black_box(resolver.create()));
            });
        }
        group.finish();
    }
}

/// Includes block_on overhead for both ready and once-pending provider futures.
fn benchmark_async_creation(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("async_creation");
    for pending in [false, true] {
        for count in [1, 8, 64] {
            let registry = AsyncProviderRegistry::<CreationSpec>::default();
            for index in 0..count {
                registry
                    .register(CreationProvider {
                        index,
                        fails: index + 1 < count,
                        pending,
                    })
                    .expect("unique provider registers");
            }
            let resolver = registry.resolve().expect("populated registry resolves");
            group.bench_function(
                BenchmarkId::new(if pending { "pending" } else { "ready" }, count),
                |bencher| {
                    bencher.iter(|| black_box(block_on(resolver.create())));
                },
            );
        }
    }
    group.finish();
}

/// Measures default-selection writes competing with snapshot reads in a bounded
/// registry.
fn benchmark_concurrent_read_write(criterion: &mut Criterion) {
    criterion.bench_function("concurrent_read_write/16_providers", |bencher| {
        bencher.iter_custom(|iterations| {
            let registry = ProviderRegistry::<CreationSpec>::default();
            for index in 0..16 {
                registry
                    .register(CreationProvider {
                        index,
                        fails: false,
                        pending: false,
                    })
                    .expect("provider registers");
            }
            let ready = Barrier::new(3);
            let start = Barrier::new(3);
            let completed = Barrier::new(3);
            thread::scope(|scope| {
                scope.spawn(|| {
                    ready.wait();
                    start.wait();
                    for index in 0..iterations {
                        registry.set_default_selection(ProviderSelection::auto().with_fallback_policy(
                            if index % 2 == 0 {
                                FallbackPolicy::Never
                            } else {
                                FallbackPolicy::OnAnyError
                            },
                        ));
                    }
                    completed.wait();
                });
                scope.spawn(|| {
                    ready.wait();
                    start.wait();
                    for _ in 0..iterations {
                        let _ = black_box(registry.resolve_default_snapshot());
                    }
                    completed.wait();
                });
                ready.wait();
                let began = Instant::now();
                start.wait();
                completed.wait();
                began.elapsed()
            })
        });
    });
}

criterion_group!(
    creation_benches,
    benchmark_creation,
    benchmark_async_creation,
    benchmark_concurrent_read_write
);
criterion_main!(creation_benches);
