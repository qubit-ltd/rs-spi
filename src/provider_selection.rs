// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Validated provider selection inputs.

use crate::internal::ProviderSelectionRepr;
use crate::{
    ProviderSelectionError, ProviderSelectionKind, ProviderSelector,
};

/// Validated request for the providers a resolver may try.
///
/// Construct selections through [`Self::auto`], [`Self::named`], or
/// [`Self::chain`]. The opaque representation prevents invalid selectors and
/// empty chains from reaching a resolver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSelection(
    /// Invariant-safe provider selection consumed by a resolver.
    ProviderSelectionRepr,
);

impl ProviderSelection {
    /// Creates an automatic provider selection.
    ///
    /// # Returns
    ///
    /// A selection using deterministic registry priority order.
    #[inline]
    #[must_use]
    pub const fn auto() -> Self {
        Self(ProviderSelectionRepr::Auto)
    }

    /// Creates a one-provider selection from configuration input.
    ///
    /// # Arguments
    ///
    /// * `value` - Raw selector normalized and validated at construction.
    ///
    /// # Returns
    ///
    /// A validated named provider selection.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderSelectionError`] when `value` cannot form a valid
    /// selector.
    pub fn named(value: impl AsRef<str>) -> Result<Self, ProviderSelectionError> {
        let input = value.as_ref();
        let selector = ProviderSelector::parse(input)
            .map_err(|source| ProviderSelectionError::invalid_selector(0, input, source))?;
        Ok(Self(ProviderSelectionRepr::Named(selector)))
    }

    /// Creates a nonempty ordered candidate chain from configuration input.
    ///
    /// # Arguments
    ///
    /// * `values` - Raw selectors normalized in encounter order.
    ///
    /// # Returns
    ///
    /// A validated nonempty selector chain preserving input order.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderSelectionError`] when any selector is invalid or when
    /// `values` contains no selectors.
    pub fn chain<I, T>(values: I) -> Result<Self, ProviderSelectionError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut selectors = Vec::new();
        for (selector_index, value) in values.into_iter().enumerate() {
            let input = value.as_ref();
            let selector = ProviderSelector::parse(input).map_err(|source| {
                ProviderSelectionError::invalid_selector(selector_index, input, source)
            })?;
            selectors.push(selector);
        }
        if selectors.is_empty() {
            return Err(ProviderSelectionError::empty_chain());
        }
        Ok(Self(ProviderSelectionRepr::Chain(
            selectors.into_boxed_slice(),
        )))
    }

    /// Returns this validated selection's classification.
    ///
    /// # Returns
    ///
    /// The automatic, named, or chained selection kind.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> ProviderSelectionKind {
        match self.0 {
            ProviderSelectionRepr::Auto => ProviderSelectionKind::Auto,
            ProviderSelectionRepr::Named(_) => ProviderSelectionKind::Named,
            ProviderSelectionRepr::Chain(_) => ProviderSelectionKind::Chain,
        }
    }

    /// Returns the named selector.
    ///
    /// # Returns
    ///
    /// `Some` for a named selection, or `None` for automatic and chained
    /// selections.
    #[inline(always)]
    #[must_use]
    pub fn selector(&self) -> Option<&ProviderSelector> {
        match &self.0 {
            ProviderSelectionRepr::Named(selector) => Some(selector),
            ProviderSelectionRepr::Auto | ProviderSelectionRepr::Chain(_) => None,
        }
    }

    /// Returns the ordered selector chain.
    ///
    /// # Returns
    ///
    /// The nonempty chain slice, or an empty slice for automatic and named
    /// selections.
    #[inline(always)]
    #[must_use]
    pub fn selectors(&self) -> &[ProviderSelector] {
        match &self.0 {
            ProviderSelectionRepr::Chain(selectors) => selectors,
            ProviderSelectionRepr::Auto | ProviderSelectionRepr::Named(_) => &[],
        }
    }

    /// Returns the validated representation consumed by the resolver.
    ///
    /// # Returns
    ///
    /// A shared reference to the invariant-safe private representation.
    #[inline(always)]
    pub(crate) const fn repr(&self) -> &ProviderSelectionRepr {
        &self.0
    }
}

impl Default for ProviderSelection {
    /// Creates the default automatic provider selection.
    #[inline(always)]
    fn default() -> Self {
        Self::auto()
    }
}
