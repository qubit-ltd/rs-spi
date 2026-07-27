// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider Registry resolution benchmarks.

use std::{
    hint::black_box,
    panic::{
        AssertUnwindSafe,
        catch_unwind,
        resume_unwind,
    },
    sync::Barrier,
    thread,
    time::Instant,
};

use criterion::{
    BenchmarkId,
    Criterion,
    Throughput,
    criterion_group,
    criterion_main,
};
use qubit_spi::error::ProviderFailure;
use qubit_spi::{
    ProviderDescriptor,
    ProviderId,
    ProviderMetadata,
    ProviderRegistry,
    ProviderSelection,
    ServiceProvider,
    ServiceSpec,
    SyncServiceSpec,
};

const NAMED_ALIAS_COUNTS: [usize; 3] = [0, 2, 8];
const CHAIN_CANDIDATE_COUNTS: [usize; 3] = [1, 8, 32];
const AUTO_PROVIDER_COUNTS: [usize; 3] = [1, 8, 64];
const REGISTRATION_PROVIDER_COUNTS: [usize; 4] = [1, 8, 64, 256];
const ALIASES_PER_PROVIDER: usize = 2;
const CONCURRENT_WORKERS: u64 = 4;

/// Provider fixture that also declares the benchmark service family.
#[derive(Debug)]
struct BenchmarkProvider {
    /// Registration metadata cloned once while the fixture is registered.
    descriptor: ProviderDescriptor,
}

impl ServiceSpec for BenchmarkProvider {
    /// Zero-sized configuration excluded from measured resolution work.
    type Config = ();
    /// Benchmark providers do not produce domain errors.
    type Error = std::io::Error;
}

impl SyncServiceSpec for BenchmarkProvider {
    /// Zero-sized output because benchmarks do not create services.
    type Output = ();
}

impl ProviderMetadata for BenchmarkProvider {
    /// Returns the descriptor snapshot used during benchmark setup.
    ///
    /// # Returns
    ///
    /// The provider's registration descriptor.
    fn descriptor(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }
}

impl ServiceProvider<BenchmarkProvider> for BenchmarkProvider {
    /// Creates the unused zero-sized benchmark output.
    ///
    /// # Parameters
    ///
    /// * `_config` - Zero-sized benchmark configuration.
    ///
    /// # Returns
    ///
    /// The zero-sized output.
    ///
    /// # Errors
    ///
    /// This fixture never returns a provider error.
    fn create_configured(
        &self,
        _config: &(),
    ) -> Result<(), ProviderFailure<std::io::Error>> {
        Ok(())
    }
}

/// Builds one canonical provider identifier.
///
/// # Parameters
///
/// * `index` - Zero-based provider position.
///
/// # Returns
///
/// A canonical identifier unique within one benchmark Registry.
fn provider_name(index: usize) -> String {
    format!("provider-{index}")
}

/// Builds one provider descriptor with deterministic aliases.
///
/// # Parameters
///
/// * `index` - Zero-based provider position.
/// * `alias_count` - Number of unique aliases to attach.
///
/// # Returns
///
/// A valid descriptor for the requested provider and alias count.
fn provider_descriptor(index: usize, alias_count: usize) -> ProviderDescriptor {
    let name = provider_name(index);
    let aliases = (0..alias_count)
        .map(|alias_index| format!("provider-{index}-alias-{alias_index}"))
        .collect::<Vec<_>>();
    ProviderDescriptor::new(
        ProviderId::new(&name).expect("benchmark provider ID should be valid"),
    )
    .with_aliases(aliases)
    .expect("benchmark provider aliases should be valid")
}

/// Builds a populated benchmark Registry outside measured iterations.
///
/// # Parameters
///
/// * `provider_count` - Number of providers to register.
/// * `alias_count` - Number of aliases assigned to each provider.
///
/// # Returns
///
/// A Registry containing providers in ascending identifier order.
fn build_registry(
    provider_count: usize,
    alias_count: usize,
) -> ProviderRegistry<BenchmarkProvider> {
    let registry = ProviderRegistry::default();
    for index in 0..provider_count {
        registry
            .register(BenchmarkProvider {
                descriptor: provider_descriptor(index, alias_count),
            })
            .expect("benchmark provider should register");
    }
    registry
}

/// Benchmarks registration while varying the number of providers.
///
/// Provider descriptors are prepared outside timed iterations so the measured
/// work isolates Registry construction, selector indexing, and priority
/// ordering.
///
/// # Parameters
///
/// * `criterion` - Criterion benchmark manager.
fn benchmark_registration(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("registration");
    for provider_count in REGISTRATION_PROVIDER_COUNTS {
        let descriptors = (0..provider_count)
            .map(|index| provider_descriptor(index, ALIASES_PER_PROVIDER))
            .collect::<Vec<_>>();
        group.throughput(Throughput::Elements(
            u64::try_from(provider_count)
                .expect("benchmark provider count should fit u64"),
        ));
        group.bench_function(
            BenchmarkId::new("register", format!("{provider_count}_providers")),
            |bencher| {
                bencher.iter(|| {
                    let registry = ProviderRegistry::default();
                    for descriptor in &descriptors {
                        registry
                            .register(BenchmarkProvider {
                                descriptor: descriptor.clone(),
                            })
                            .expect("benchmark provider should register");
                    }
                    black_box(registry);
                });
            },
        );
    }
    group.finish();
}

/// Benchmarks named resolution while varying descriptor alias count.
///
/// # Parameters
///
/// * `criterion` - Criterion benchmark manager.
fn benchmark_named_resolution(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("named_resolution");
    for alias_count in NAMED_ALIAS_COUNTS {
        let registry = build_registry(8, alias_count);
        let selection = ProviderSelection::named("provider-0")
            .expect("benchmark named selection should be valid");
        group.bench_function(
            BenchmarkId::new("resolve", format!("{alias_count}_aliases")),
            |bencher| {
                bencher.iter(|| {
                    black_box(
                        registry
                            .resolve_selected(black_box(&selection))
                            .expect("benchmark named selection should resolve"),
                    )
                });
            },
        );
    }
    group.finish();
}

/// Benchmarks chained resolution while varying candidate count.
///
/// # Parameters
///
/// * `criterion` - Criterion benchmark manager.
fn benchmark_chain_resolution(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("chain_resolution");
    for candidate_count in CHAIN_CANDIDATE_COUNTS {
        let registry = build_registry(candidate_count, ALIASES_PER_PROVIDER);
        let provider_names =
            (0..candidate_count).map(provider_name).collect::<Vec<_>>();
        let selection =
            ProviderSelection::chain(provider_names.iter().map(String::as_str))
                .expect("benchmark chain selection should be valid");
        group.bench_function(
            BenchmarkId::new(
                "resolve",
                format!("{candidate_count}_candidates"),
            ),
            |bencher| {
                bencher.iter(|| {
                    black_box(
                        registry
                            .resolve_selected(black_box(&selection))
                            .expect("benchmark chain selection should resolve"),
                    )
                });
            },
        );
    }
    group.finish();
}

/// Benchmarks automatic resolution while varying registered provider count.
///
/// # Parameters
///
/// * `criterion` - Criterion benchmark manager.
fn benchmark_auto_resolution(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("auto_resolution");
    for provider_count in AUTO_PROVIDER_COUNTS {
        let registry = build_registry(provider_count, ALIASES_PER_PROVIDER);
        let selection = ProviderSelection::auto();
        group.bench_function(
            BenchmarkId::new("resolve", format!("{provider_count}_providers")),
            |bencher| {
                bencher.iter(|| {
                    black_box(
                        registry
                            .resolve_selected(black_box(&selection))
                            .expect(
                                "benchmark automatic selection should resolve",
                            ),
                    )
                });
            },
        );
    }
    group.finish();
}

/// Benchmarks concurrent automatic resolution against one shared Registry.
///
/// Worker threads reach a readiness barrier before timing starts. The returned
/// duration excludes thread creation and scoped-thread joining.
///
/// # Parameters
///
/// * `criterion` - Criterion benchmark manager.
fn benchmark_concurrent_resolution(criterion: &mut Criterion) {
    let registry = build_registry(16, ALIASES_PER_PROVIDER);
    let selection = ProviderSelection::auto();
    let worker_count = usize::try_from(CONCURRENT_WORKERS)
        .expect("benchmark worker count should fit usize");
    let mut group = criterion.benchmark_group("concurrent_resolution");
    group.throughput(Throughput::Elements(CONCURRENT_WORKERS));
    group.bench_function("4_workers_16_providers", |bencher| {
        bencher.iter_custom(|iterations| {
            let participant_count = worker_count + 1;
            let ready = Barrier::new(participant_count);
            let start = Barrier::new(participant_count);
            let completed = Barrier::new(participant_count);
            thread::scope(|scope| {
                for _ in 0..worker_count {
                    let ready = &ready;
                    let start = &start;
                    let completed = &completed;
                    let registry = &registry;
                    let selection = &selection;
                    scope.spawn(move || {
                        ready.wait();
                        let outcome = catch_unwind(AssertUnwindSafe(|| {
                            start.wait();
                            for _ in 0..iterations {
                                black_box(
                                    registry
                                        .resolve_selected(black_box(selection))
                                        .expect("concurrent automatic selection should resolve"),
                                );
                            }
                        }));
                        completed.wait();
                        if let Err(payload) = outcome {
                            resume_unwind(payload);
                        }
                    });
                }
                ready.wait();
                let started_at = Instant::now();
                start.wait();
                completed.wait();
                started_at.elapsed()
            })
        });
    });
    group.finish();
}

criterion_group!(
    registry_resolution_benches,
    benchmark_registration,
    benchmark_named_resolution,
    benchmark_chain_resolution,
    benchmark_auto_resolution,
    benchmark_concurrent_resolution,
);
criterion_main!(registry_resolution_benches);
