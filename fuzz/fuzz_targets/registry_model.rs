// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Compares registration, full fallback traversal and historical snapshots to a
//! linear model.
#![no_main]

use std::io::Error;
use std::sync::Arc;
use std::sync::Mutex;

use libfuzzer_sys::fuzz_target;
use qubit_spi::FallbackPolicy;
use qubit_spi::ProviderCreationTermination;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::ProviderMetadata;
use qubit_spi::ProviderRegistry;
use qubit_spi::ProviderSelection;
use qubit_spi::ResolvingServiceProvider;
use qubit_spi::ServiceProvider;
use qubit_spi::ServiceSpec;
use qubit_spi::SyncServiceSpec;
use qubit_spi::error::ProviderFailure;
use qubit_spi::error::ProviderFailureKind;
use qubit_spi::error::ProviderResolutionError;

const MAX_REGISTRATIONS: usize = 32;
const MAX_SELECTORS: usize = 16;
const MAX_OPERATIONS: usize = 32;

/// The model performs no filesystem or network operations.
struct FuzzSpec;

impl ServiceSpec for FuzzSpec {
    type Config = ();
    type Error = Error;
}

impl SyncServiceSpec for FuzzSpec {
    type Output = String;
}

/// Linear model entry; ownership lookup deliberately uses no production-style
/// index.
#[derive(Clone)]
struct ModelEntry {
    id: String,
    alias: Option<String>,
    priority: i32,
    outcome: u8,
}

/// Actual provider records every invocation, including failures preceding
/// success.
struct FuzzProvider {
    entry: ModelEntry,
    calls: Arc<Mutex<Vec<String>>>,
}

impl ProviderMetadata for FuzzProvider {
    /// Builds immutable registration metadata from validated bounded input.
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(ProviderId::new(&self.entry.id).expect("valid model ID"))
            .with_priority(self.entry.priority)
            .with_aliases(self.entry.alias.iter())
            .expect("valid model alias")
    }
}

impl ServiceProvider<FuzzSpec> for FuzzProvider {
    /// Returns the input-selected success or typed failure after recording
    /// invocation.
    fn create_configured(&self, _config: &()) -> Result<String, ProviderFailure<Error>> {
        self.calls
            .lock()
            .expect("single-threaded event lock")
            .push(self.entry.id.clone());
        let error = Error::other("model failure");
        match self.entry.outcome {
            0 => Ok(self.entry.id.clone()),
            1 => Err(ProviderFailure::unsupported(error)),
            2 => Err(ProviderFailure::unavailable(error)),
            3 => Err(ProviderFailure::invalid_configuration(error)),
            4 => Err(ProviderFailure::initialization_failed(error)),
            _ => unreachable!("bounded outcome"),
        }
    }
}

/// Model-side target discriminant is independent of the production selection
/// representation.
#[derive(Clone)]
struct Request {
    mode: u8,
    selectors: Vec<String>,
    policy: u8,
}

impl Request {
    /// Constructs the corresponding public request without using it as the
    /// oracle.
    fn selection(&self) -> ProviderSelection {
        let selection = match self.mode {
            0 => ProviderSelection::auto(),
            1 => ProviderSelection::named(&self.selectors[0]).expect("valid selector"),
            2 => ProviderSelection::chain(&self.selectors).expect("valid nonempty selectors"),
            3 => ProviderSelection::chain_allowing_missing(&self.selectors).expect("valid nonempty selectors"),
            _ => unreachable!("bounded mode"),
        };
        selection.with_fallback_policy(match self.policy {
            0 => FallbackPolicy::Never,
            1 => FallbackPolicy::OnAbsence,
            2 => FallbackPolicy::OnAnyError,
            _ => unreachable!("bounded policy"),
        })
    }
}

/// Finds ownership by scanning canonical IDs and aliases in registration order.
fn owner<'a>(entries: &'a [ModelEntry], selector: &str) -> Option<&'a ModelEntry> {
    entries
        .iter()
        .find(|entry| entry.id == selector || entry.alias.as_deref() == Some(selector))
}

/// Computes the candidate list or resolution failure independently of
/// production helpers.
fn candidates(entries: &[ModelEntry], request: &Request) -> Result<Vec<ModelEntry>, (u8, Vec<String>)> {
    if request.mode == 0 {
        if entries.is_empty() {
            return Err((0, vec![]));
        }
        let mut sorted = entries.to_vec();
        sorted.sort_by(|left, right| right.priority.cmp(&left.priority).then_with(|| left.id.cmp(&right.id)));
        return Ok(sorted);
    }
    let selectors = if request.mode == 1 {
        &request.selectors[..1]
    } else {
        &request.selectors
    };
    let mut missing = Vec::new();
    let mut selected: Vec<ModelEntry> = Vec::new();
    for selector in selectors {
        if let Some(entry) = owner(entries, selector) {
            if !selected.iter().any(|previous| previous.id == entry.id) {
                selected.push(entry.clone());
            }
        } else {
            missing.push(selector.clone());
        }
    }
    if request.mode != 3 && !missing.is_empty() {
        return Err((1, missing));
    }
    if selected.is_empty() {
        return Err((2, selectors.to_vec()));
    }
    Ok(selected)
}

/// Validates full invocation and diagnostic history, including the final
/// failure rule.
fn assert_creation(
    resolver: &ResolvingServiceProvider<FuzzSpec>,
    entries: &[ModelEntry],
    request: &Request,
    calls: &Mutex<Vec<String>>,
) {
    calls.lock().expect("event lock").clear();
    let result = resolver.create();
    let mut expected_calls = Vec::new();
    let mut attempts = Vec::new();
    let mut success = None;
    let mut termination = ProviderCreationTermination::Exhausted;
    for (index, entry) in entries.iter().enumerate() {
        expected_calls.push(entry.id.clone());
        if entry.outcome == 0 {
            success = Some(entry.id.clone());
            break;
        }
        let kind = match entry.outcome {
            1 => ProviderFailureKind::Unsupported,
            2 => ProviderFailureKind::Unavailable,
            3 => ProviderFailureKind::InvalidConfiguration,
            4 => ProviderFailureKind::InitializationFailed,
            _ => unreachable!("bounded outcome"),
        };
        attempts.push((entry.id.clone(), kind));
        if index + 1 == entries.len() {
            break;
        }
        // Do not call FallbackPolicy::should_continue_after in this oracle.
        if request.policy == 0 || (request.policy == 1 && entry.outcome > 2) {
            termination = ProviderCreationTermination::StoppedByPolicy;
            break;
        }
    }
    assert_eq!(expected_calls, *calls.lock().expect("event lock"));
    if let Some(expected) = success {
        assert_eq!(expected, result.expect("model predicts success"));
    } else {
        let error = result.expect_err("model predicts failure");
        assert_eq!(termination, error.termination());
        assert_eq!(
            attempts,
            error
                .attempts()
                .iter()
                .map(|attempt| (attempt.provider_id().as_str().to_owned(), attempt.failure().kind()))
                .collect::<Vec<_>>()
        );
    }
}

/// Compares structured resolution errors rather than merely checking is_err.
fn assert_resolution_error(error: ProviderResolutionError, expected: (u8, Vec<String>)) {
    let (kind, selectors) = expected;
    match kind {
        0 => assert!(matches!(error, ProviderResolutionError::EmptyRegistry)),
        1 => assert!(matches!(error, ProviderResolutionError::UnknownProviders { .. })),
        2 => assert!(matches!(error, ProviderResolutionError::NoCandidates { .. })),
        _ => unreachable!("bounded resolution error"),
    }
    assert_eq!(
        selectors,
        error
            .selectors()
            .unwrap_or_default()
            .iter()
            .map(|selector| selector.as_str().to_owned())
            .collect::<Vec<_>>()
    );
}

/// Creates deliberate canonical/alias collisions and unknown selectors.
fn selector(value: u8) -> String {
    format!("{}-{}", if value & 128 == 0 { "provider" } else { "alias" }, value % 16)
}

fuzz_target!(|data: &[u8]| {
    let registry = ProviderRegistry::<FuzzSpec>::default();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut entries = Vec::new();
    let mut historical = Vec::new();
    let selectors = data
        .iter()
        .rev()
        .take(MAX_SELECTORS)
        .map(|value| selector(*value))
        .collect::<Vec<_>>();
    let selectors = if selectors.is_empty() {
        vec!["unknown".to_owned()]
    } else {
        selectors
    };
    for (operation, fields) in data.chunks_exact(5).take(MAX_OPERATIONS).enumerate() {
        if operation < MAX_REGISTRATIONS {
            let id = selector(fields[0] & 127);
            let alias = (fields[1] != 255)
                .then(|| selector(fields[1]))
                .filter(|alias| *alias != id);
            let priority = match fields[2] {
                0 => i32::MIN,
                255 => i32::MAX,
                value => i32::from(value as i8),
            };
            let entry = ModelEntry {
                id,
                alias,
                priority,
                outcome: fields[3] % 5,
            };
            let expected = owner(&entries, &entry.id).is_none()
                && entry
                    .alias
                    .as_ref()
                    .is_none_or(|alias| owner(&entries, alias).is_none());
            let result = registry.register(FuzzProvider {
                entry: entry.clone(),
                calls: Arc::clone(&calls),
            });
            assert_eq!(expected, result.is_ok());
            if expected {
                entries.push(entry);
            }
            assert_eq!(
                entries.iter().map(|entry| entry.id.as_str()).collect::<Vec<_>>(),
                registry
                    .provider_ids()
                    .iter()
                    .map(ProviderId::as_str)
                    .collect::<Vec<_>>()
            );
            assert_eq!(entries.len(), registry.len());
        }
        // Every mode is exercised at each operation, including strict multi-missing
        // chains.
        for mode in 0..4 {
            let request = Request {
                mode,
                selectors: selectors.clone(),
                policy: fields[4] % 3,
            };
            let selection = request.selection();
            registry.set_default_selection(selection.clone());
            let (captured_selection, actual) = registry.resolve_default_snapshot();
            assert_eq!(selection, captured_selection);
            match (actual, candidates(&entries, &request)) {
                (Ok(resolver), Ok(expected)) => {
                    assert_creation(&resolver, &expected, &request, &calls);
                    if mode == fields[4] % 4 {
                        historical.push((resolver, expected, request));
                    }
                }
                (Err(error), Err(expected)) => assert_resolution_error(error, expected),
                _ => panic!("model and registry disagree about resolution"),
            }
        }
    }
    if data.len() < 5 {
        assert!(matches!(
            registry.resolve(),
            Err(ProviderResolutionError::EmptyRegistry)
        ));
    }
    // Replaying captured candidates after all later registrations/default changes
    // must be stable.
    for (resolver, expected, request) in historical {
        assert_creation(&resolver, &expected, &request, &calls);
    }
});
