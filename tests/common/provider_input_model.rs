// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Independent standard-library model for provider input normalization.

/// Checks canonical grammar independently of production parsing.
pub(crate) fn canonical_token(input: &str) -> bool {
    fn endpoint(c: char) -> bool {
        matches!(c, 'a'..='z' | '0'..='9')
    }

    input.chars().next().is_some_and(endpoint)
        && input.chars().next_back().is_some_and(endpoint)
        && input.chars().all(|c| endpoint(c) || "-_.+".contains(c))
}

/// Predicts a normalized selector without production constructors.
pub(crate) fn selector(input: &str) -> Option<String> {
    let normalized: String = input
        .trim_matches(char::is_whitespace)
        .chars()
        .map(|c| c.to_ascii_lowercase())
        .collect();
    canonical_token(&normalized).then_some(normalized)
}

/// Predicts full alias acceptance and normalized order.
pub(crate) fn aliases(id: &str, inputs: &[&str]) -> Option<Vec<String>> {
    let mut normalized = Vec::new();
    for input in inputs {
        let alias = selector(input)?;
        if alias == id || normalized.contains(&alias) {
            return None;
        }
        normalized.push(alias);
    }
    Some(normalized)
}
