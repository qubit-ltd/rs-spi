// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzzes provider Registry indexing and selection against a reference model.

#![no_main]

use std::collections::{
    HashMap,
    HashSet,
};

use libfuzzer_sys::fuzz_target;
use qubit_spi::error::ProviderFailure;
use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderMetadata,
    ProviderRegistry,
    ProviderSelection,
    ServiceProvider,
    ServiceSpec,
    SyncServiceSpec,
};

/// Upper bound for provider registrations derived from one fuzzer input.
const MAX_REGISTRATIONS: usize = 32;
/// Upper bound for selectors resolved from one fuzzer input.
const MAX_SELECTORS: usize = 16;

/// Service family used only by the Registry fuzz fixture.
struct FuzzSpec;

impl ServiceSpec for FuzzSpec {
    /// Configuration is unused by the provider fixtures.
    type Config = ();
    /// Fuzz providers do not produce domain errors.
    type Error = std::io::Error;
}

impl SyncServiceSpec for FuzzSpec {
    /// Successful providers return their canonical identifier.
    type Output = String;
}

/// Registered provider that returns its canonical identifier.
struct FuzzProvider {
    /// Immutable registration metadata.
    descriptor: ProviderDescriptor,
}

impl ProviderMetadata for FuzzProvider {
    /// Returns the provider's immutable registration metadata.
    fn descriptor(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }
}

impl ServiceProvider<FuzzSpec> for FuzzProvider {
    /// Returns the canonical identifier selected by the Registry.
    ///
    /// # Parameters
    ///
    /// * `_config` - Unused zero-sized fuzz configuration.
    ///
    /// # Returns
    ///
    /// The registered provider's canonical identifier.
    ///
    /// # Errors
    ///
    /// This fixture never returns a provider creation error.
    fn create_configured(
        &self,
        _config: &(),
    ) -> Result<String, ProviderFailure<std::io::Error>> {
        Ok(self.descriptor.id().as_str().to_owned())
    }
}

/// Minimal model of successful Registry registrations and selector ownership.
#[derive(Default)]
struct RegistryModel {
    /// Canonical provider IDs in successful registration order.
    registration_ids: Vec<String>,
    /// Canonical provider priority indexed by identifier.
    priorities: HashMap<String, i32>,
    /// Canonical IDs and aliases indexed by their owning provider ID.
    selector_ids: HashMap<String, String>,
}

impl RegistryModel {
    /// Reports whether a descriptor can register without selector conflicts.
    ///
    /// # Parameters
    ///
    /// * `provider_id` - Candidate canonical provider identifier.
    /// * `alias` - Optional candidate selector alias.
    ///
    /// # Returns
    ///
    /// `true` when neither selector is already owned.
    fn can_register(&self, provider_id: &str, alias: Option<&str>) -> bool {
        !self.selector_ids.contains_key(provider_id)
            && alias.is_none_or(|alias| !self.selector_ids.contains_key(alias))
    }

    /// Records one registration previously accepted by the concrete Registry.
    ///
    /// # Parameters
    ///
    /// * `provider_id` - Newly registered canonical provider identifier.
    /// * `alias` - Optional registered selector alias.
    /// * `priority` - Automatic-selection priority.
    fn register(
        &mut self,
        provider_id: String,
        alias: Option<String>,
        priority: i32,
    ) {
        self.selector_ids
            .insert(provider_id.clone(), provider_id.clone());
        if let Some(alias) = alias {
            self.selector_ids.insert(alias, provider_id.clone());
        }
        self.priorities.insert(provider_id.clone(), priority);
        self.registration_ids.push(provider_id);
    }

    /// Returns the canonical ID selected first by automatic selection.
    ///
    /// # Returns
    ///
    /// The highest-priority canonical ID, breaking ties lexicographically, or
    /// `None` when the model has no registered providers.
    fn first_auto_id(&self) -> Option<&str> {
        self.priorities
            .iter()
            .min_by(|(left_id, left_priority), (right_id, right_priority)| {
                right_priority
                    .cmp(left_priority)
                    .then_with(|| left_id.cmp(right_id))
            })
            .map(|(provider_id, _)| provider_id.as_str())
    }

    /// Resolves selectors in chain order while suppressing duplicate providers.
    ///
    /// # Parameters
    ///
    /// * `selectors` - Valid canonical selectors in requested chain order.
    ///
    /// # Returns
    ///
    /// Canonical provider IDs in the order a lenient chain attempts them.
    fn resolve_lenient_chain(&self, selectors: &[String]) -> Vec<String> {
        let mut seen = HashSet::with_capacity(selectors.len());
        let mut candidates = Vec::with_capacity(selectors.len());
        for selector in selectors {
            let Some(provider_id) = self.selector_ids.get(selector) else {
                continue;
            };
            if seen.insert(provider_id) {
                candidates.push(provider_id.clone());
            }
        }
        candidates
    }
}

/// Maps one byte to a bounded canonical provider identifier.
///
/// # Parameters
///
/// * `value` - Byte used to choose one fixture provider identifier.
///
/// # Returns
///
/// A valid canonical provider identifier shared by controlled collisions.
fn provider_id(value: u8) -> String {
    format!("provider-{}", value % 8)
}

/// Maps one byte to a bounded canonical selector alias.
///
/// # Parameters
///
/// * `value` - Byte used to choose one fixture alias.
///
/// # Returns
///
/// A valid canonical alias shared by controlled collisions.
fn alias(value: u8) -> String {
    format!("alias-{}", value % 8)
}

/// Builds a valid self-described provider from one model registration.
///
/// # Parameters
///
/// * `provider_id` - Canonical identifier for the provider.
/// * `alias` - Optional selector alias.
/// * `priority` - Automatic-selection priority.
///
/// # Returns
///
/// A provider fixture with the requested descriptor.
fn provider(
    provider_id: &str,
    alias: Option<&str>,
    priority: i32,
) -> FuzzProvider {
    let provider_id = ProviderId::new(provider_id)
        .expect("bounded fuzz provider IDs must be canonical");
    let descriptor =
        ProviderDescriptor::new(provider_id).with_priority(priority);
    let descriptor = match alias {
        Some(alias) => descriptor
            .with_aliases([alias])
            .expect("bounded fuzz aliases must be valid and distinct"),
        None => descriptor,
    };
    FuzzProvider { descriptor }
}

/// Checks registration metadata against the reference model.
///
/// # Parameters
///
/// * `registry` - Concrete Registry under fuzzing.
/// * `model` - Reference model built from accepted registrations.
fn assert_registration_model(
    registry: &ProviderRegistry<FuzzSpec>,
    model: &RegistryModel,
) {
    let registered_ids = registry.provider_ids();
    let actual_ids = registered_ids
        .iter()
        .map(ProviderId::as_str)
        .collect::<Vec<_>>();
    assert_eq!(model.registration_ids, actual_ids);
    assert_eq!(model.registration_ids.len(), registry.len());
}

/// Checks named, chain, and automatic selection against the reference model.
///
/// # Parameters
///
/// * `registry` - Concrete Registry under fuzzing.
/// * `model` - Reference model built from accepted registrations.
/// * `selectors` - Valid selectors chosen from the fuzzer input.
fn assert_resolution_model(
    registry: &ProviderRegistry<FuzzSpec>,
    model: &RegistryModel,
    selectors: &[String],
) {
    let lenient_selection = ProviderSelection::chain_allowing_missing(
        selectors.iter().map(String::as_str),
    )
    .expect("the fuzzer always creates a nonempty selector chain");
    let expected_chain = model.resolve_lenient_chain(selectors);
    match registry.resolve_selected(&lenient_selection) {
        Ok(resolver) => {
            let actual = resolver
                .create()
                .expect("the fuzz provider fixture must create successfully");
            assert_eq!(expected_chain.first(), Some(&actual));
        }
        Err(_) => assert!(expected_chain.is_empty()),
    }

    let strict_selection =
        ProviderSelection::chain(selectors.iter().map(String::as_str))
            .expect("the fuzzer always creates a nonempty selector chain");
    let has_missing = selectors
        .iter()
        .any(|selector| !model.selector_ids.contains_key(selector));
    match registry.resolve_selected(&strict_selection) {
        Ok(resolver) => {
            assert!(!has_missing);
            let actual = resolver
                .create()
                .expect("the fuzz provider fixture must create successfully");
            assert_eq!(expected_chain.first(), Some(&actual));
        }
        Err(_) => assert!(has_missing || expected_chain.is_empty()),
    }

    let auto_selection = ProviderSelection::auto()
        .with_fallback_policy(FallbackPolicy::OnAnyError);
    match registry.resolve_selected(&auto_selection) {
        Ok(resolver) => {
            let actual = resolver
                .create()
                .expect("the fuzz provider fixture must create successfully");
            assert_eq!(model.first_auto_id(), Some(actual.as_str()));
        }
        Err(_) => assert!(model.registration_ids.is_empty()),
    }
}

fuzz_target!(|data: &[u8]| {
    let registry = ProviderRegistry::<FuzzSpec>::default();
    let mut model = RegistryModel::default();

    for fields in data.chunks_exact(3).take(MAX_REGISTRATIONS) {
        let provider_id = provider_id(fields[0]);
        let alias = (fields[1] & 1 == 0).then(|| alias(fields[1]));
        let priority = i32::from(fields[2] as i8);
        let expected = model.can_register(&provider_id, alias.as_deref());
        let actual = registry.register(provider(
            &provider_id,
            alias.as_deref(),
            priority,
        ));
        assert_eq!(expected, actual.is_ok());
        if expected {
            model.register(provider_id, alias, priority);
        }
        assert_registration_model(&registry, &model);
    }

    let selectors = data
        .iter()
        .rev()
        .take(MAX_SELECTORS)
        .map(|value| {
            if value % 3 == 0 {
                format!("missing-{}", value % 8)
            } else if value & 1 == 0 {
                alias(*value)
            } else {
                provider_id(*value)
            }
        })
        .collect::<Vec<_>>();
    let selectors = if selectors.is_empty() {
        vec!["missing-0".to_owned()]
    } else {
        selectors
    };
    assert_resolution_model(&registry, &model, &selectors);
});
