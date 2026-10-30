// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ProviderDescriptor;
use qubit_spi::ProviderId;
use qubit_spi::ProviderSelection;
use qubit_spi::ProviderSelectionTargetRef;
use qubit_spi::ProviderSelector;

use crate::common::provider_input_model as model;

#[test]
fn test_input_model_boundaries() {
    for (input, canonical, normalized) in [
        ("a", true, Some("a")),
        ("0", true, Some("0")),
        ("a--b", true, Some("a--b")),
        ("a..b", true, Some("a..b")),
        ("git+ssh", true, Some("git+ssh")),
        ("", false, None),
        ("A", false, Some("a")),
        ("-a", false, None),
        ("a-", false, None),
        ("a/b", false, None),
        ("a\0b", false, None),
        ("é", false, None),
        (" FILE-Command ", false, Some("file-command")),
        ("\u{2003}A\u{2003}", false, Some("a")),
        ("a b", false, None),
        ("\t ", false, None),
    ] {
        assert_eq!(canonical, model::canonical_token(input), "{input:?}");
        assert_eq!(normalized.map(str::to_owned), model::selector(input), "{input:?}");
        assert_eq!(canonical, ProviderId::new(input).is_ok(), "{input:?}");
        assert_eq!(
            normalized.map(str::to_owned),
            ProviderSelector::parse(input)
                .ok()
                .map(|selector| selector.as_str().to_owned()),
            "{input:?}"
        );
    }
}

#[test]
fn test_input_model_aliases() {
    let cases: &[(&[&str], Option<Vec<&str>>)] = &[
        (&[], Some(vec![])),
        (&[" B ", "a"], Some(vec!["b", "a"])),
        (&["A", " a "], None),
        (&[" FUZZ-PROVIDER "], None),
        (&["valid", "bad/alias"], None),
    ];
    for (inputs, expected) in cases {
        let expected = expected
            .as_ref()
            .map(|values| values.iter().map(|value| (*value).to_owned()).collect::<Vec<_>>());
        assert_eq!(expected, model::aliases("fuzz-provider", inputs));
        let actual = ProviderDescriptor::new(ProviderId::new("fuzz-provider").expect("valid fixed ID"))
            .with_aliases(inputs.iter().copied())
            .ok()
            .map(|descriptor| {
                descriptor
                    .aliases()
                    .iter()
                    .map(|alias| alias.as_str().to_owned())
                    .collect::<Vec<_>>()
            });
        assert_eq!(expected, actual);
    }
}

#[test]
fn test_input_model_chains() {
    let cases: &[(&[&str], Option<Vec<&str>>)] = &[
        (&[], None),
        (&[" A ", "a", "b"], Some(vec!["a", "a", "b"])),
        (&["a", "bad/value"], None),
    ];
    for (inputs, expected) in cases {
        let expected = expected
            .as_ref()
            .map(|values| values.iter().map(|value| (*value).to_owned()).collect::<Vec<_>>());
        let modeled = if inputs.is_empty() {
            None
        } else {
            inputs
                .iter()
                .map(|input| model::selector(input))
                .collect::<Option<Vec<_>>>()
        };
        assert_eq!(expected, modeled);
        let actual = ProviderSelection::chain(inputs.iter().copied())
            .ok()
            .map(|selection| match selection.target() {
                ProviderSelectionTargetRef::Chain { selectors, .. } => selectors
                    .iter()
                    .map(|selector| selector.as_str().to_owned())
                    .collect::<Vec<_>>(),
                _ => panic!("expected chain"),
            });
        assert_eq!(expected, actual);
    }
}
