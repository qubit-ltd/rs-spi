// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fuzzes provider identity, selector, alias, and strict-chain invariants.

#![no_main]

use std::collections::BTreeSet;

use libfuzzer_sys::fuzz_target;
use qubit_spi::MissingProviderPolicy;
use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::ProviderSelection;
use qubit_spi::ProviderSelectionTargetRef;
use qubit_spi::ProviderSelector;

/// Canonical provider ID reserved for descriptor fuzz fixtures.
const FUZZ_PROVIDER_ID: &str = "fuzz-provider";
/// Maximum number of alias or chain fields derived from one input.
const MAX_FIELDS: usize = 16;
/// Maximum input size aligned with the CI fuzzing limit to cap parsing work.
const MAX_INPUT_BYTES: usize = 4096;

/// Verifies successful identity and selector parsing remains canonical.
///
/// # Parameters
///
/// * `input` - Complete UTF-8 fuzzer input.
fn assert_identity_invariants(input: &str) {
    if let Ok(id) = ProviderId::new(input) {
        assert_eq!(input, id.as_str());
        let selector = ProviderSelector::parse(id.as_str()).expect("a canonical provider ID must parse as a selector");
        assert_eq!(id.as_str(), selector.as_str());
    }

    match ProviderSelector::parse(input) {
        Ok(selector) => {
            let reparsed = ProviderSelector::parse(selector.as_str()).expect("a normalized selector must parse again");
            assert_eq!(selector, reparsed);
            let id = ProviderId::new(selector.as_str()).expect("a normalized selector must be a canonical provider ID");
            assert_eq!(selector.as_str(), id.as_str());
            let selection = ProviderSelection::named(input).expect("a valid selector must form a named selection");
            assert!(matches!(
                selection.target(),
                ProviderSelectionTargetRef::Named(actual)
                    if actual == &selector
            ));
        }
        Err(_) => assert!(ProviderSelection::named(input).is_err()),
    }
}

/// Verifies successful alias normalization preserves public invariants.
///
/// # Parameters
///
/// * `inputs` - Bounded raw alias fields in caller-supplied order.
fn assert_descriptor_invariants(inputs: &[&str]) {
    let provider_id = ProviderId::new(FUZZ_PROVIDER_ID).expect("the fixed fuzz provider ID must be canonical");
    let Ok(descriptor) = ProviderDescriptor::new(provider_id).with_aliases(inputs.iter().copied()) else {
        return;
    };

    assert_eq!(FUZZ_PROVIDER_ID, descriptor.id().as_str());
    assert_eq!(inputs.len(), descriptor.aliases().len());
    let mut unique_aliases = BTreeSet::new();
    for (input, alias) in inputs.iter().zip(descriptor.aliases()) {
        let expected = ProviderSelector::parse(input).expect("a successful descriptor must contain valid aliases");
        assert_eq!(&expected, alias);
        assert_ne!(FUZZ_PROVIDER_ID, alias.as_str());
        assert!(unique_aliases.insert(alias.as_str()));
    }
}

/// Verifies strict-chain construction agrees with selector parsing.
///
/// # Parameters
///
/// * `inputs` - Bounded raw selector fields in caller-supplied order.
fn assert_chain_invariants(inputs: &[&str]) {
    let selection = ProviderSelection::chain(inputs.iter().copied());
    let expected = inputs
        .iter()
        .map(|input| ProviderSelector::parse(input))
        .collect::<Result<Vec<_>, _>>();
    let Ok(expected) = expected else {
        assert!(selection.is_err());
        return;
    };
    if expected.is_empty() {
        assert!(selection.is_err());
        return;
    }

    let selection = selection.expect("a nonempty list of valid selectors must form a chain");
    match selection.target() {
        ProviderSelectionTargetRef::Chain {
            selectors,
            missing_policy: MissingProviderPolicy::Reject,
        } => assert_eq!(expected.as_slice(), selectors),
        _ => panic!("a strict chain must retain a strict-chain target"),
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_BYTES {
        return;
    }
    let Ok(input) = str::from_utf8(data) else {
        return;
    };
    assert_identity_invariants(input);
    let fields = input.split('\n').take(MAX_FIELDS).collect::<Vec<_>>();
    assert_descriptor_invariants(&fields);
    assert_chain_invariants(&fields);
});
