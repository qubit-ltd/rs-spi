// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private parsing helpers for validated provider selections.

use crate::ProviderSelector;
use crate::error::ProviderSelectionBuildError;

/// Parses selector inputs and rejects an empty chain.
///
/// Each selector is normalized in encounter order. Invalid input reports its
/// zero-based position, while an empty input reports the chain-specific error.
///
/// # Type Parameters
///
/// * `I` - Iterator-like source of selector inputs.
/// * `T` - Individual selector input convertible to a string reference.
///
/// # Parameters
///
/// * `values` - Raw selector inputs to normalize in encounter order.
///
/// # Returns
///
/// A nonempty boxed slice of normalized selectors.
///
/// # Errors
///
/// Returns [`ProviderSelectionBuildError`] when an input is invalid or when
/// `values` contains no selectors.
pub(in crate::selection) fn parse_chain_selectors<I, T>(
    values: I,
) -> Result<Box<[ProviderSelector]>, ProviderSelectionBuildError>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let mut selectors = Vec::new();
    for (selector_index, value) in values.into_iter().enumerate() {
        let input = value.as_ref();
        let selector = ProviderSelector::parse(input)
            .map_err(|source| ProviderSelectionBuildError::invalid_selector(Some(selector_index), source))?;
        selectors.push(selector);
    }
    if selectors.is_empty() {
        return Err(ProviderSelectionBuildError::empty_chain());
    }
    Ok(selectors.into_boxed_slice())
}
