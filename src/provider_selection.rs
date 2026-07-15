// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Explicit provider selection inputs and fallback policy.

use crate::{ProviderSelectionError, ProviderSelector};

/// Classification of a validated provider selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderSelectionKind {
    /// Providers are tried in deterministic automatic order.
    Auto,
    /// Exactly one normalized selector is used.
    Named,
    /// Normalized selectors are tried in caller-supplied order.
    Chain,
}

/// Validated request for the providers a resolver may try.
///
/// Construct selections through [`Self::auto`], [`Self::named`], or
/// [`Self::chain`]. The opaque representation prevents invalid selectors and
/// empty chains from reaching a resolver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSelection(ProviderSelectionRepr);

/// Private invariant-safe provider selection representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProviderSelectionRepr {
    /// Providers are tried in deterministic automatic order.
    Auto,
    /// Exactly one normalized selector is used.
    Named(ProviderSelector),
    /// A nonempty ordered selector chain is used.
    Chain(Box<[ProviderSelector]>),
}

impl ProviderSelection {
    /// Creates an automatic provider selection.
    #[must_use]
    pub const fn auto() -> Self {
        Self(ProviderSelectionRepr::Auto)
    }

    /// Creates a one-provider selection from configuration input.
    ///
    /// `value` is normalized as a provider selector. Returns a validated named
    /// selection containing that selector.
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
    /// Each item in `values` is normalized as a provider selector. Returns a
    /// validated chain preserving selector order.
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
    #[must_use]
    pub const fn kind(&self) -> ProviderSelectionKind {
        match self.0 {
            ProviderSelectionRepr::Auto => ProviderSelectionKind::Auto,
            ProviderSelectionRepr::Named(_) => ProviderSelectionKind::Named,
            ProviderSelectionRepr::Chain(_) => ProviderSelectionKind::Chain,
        }
    }

    /// Returns the named selector, when this is a named selection.
    #[must_use]
    pub fn selector(&self) -> Option<&ProviderSelector> {
        match &self.0 {
            ProviderSelectionRepr::Named(selector) => Some(selector),
            ProviderSelectionRepr::Auto | ProviderSelectionRepr::Chain(_) => None,
        }
    }

    /// Returns the ordered selector chain, or an empty slice for other kinds.
    #[must_use]
    pub fn selectors(&self) -> &[ProviderSelector] {
        match &self.0 {
            ProviderSelectionRepr::Chain(selectors) => selectors,
            ProviderSelectionRepr::Auto | ProviderSelectionRepr::Named(_) => &[],
        }
    }

    /// Returns the private representation used by the resolver.
    pub(crate) const fn repr(&self) -> &ProviderSelectionRepr {
        &self.0
    }
}

impl Default for ProviderSelection {
    /// Creates the default automatic provider selection.
    fn default() -> Self {
        Self::auto()
    }
}

/// Controls which provider creation failures permit trying another candidate.
///
/// This policy applies to automatic and chained selection after a provider
/// factory returns an error. Named selection always uses exactly one provider
/// and never falls back.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FallbackPolicy {
    /// Continues only after an unsupported or unavailable provider.
    #[default]
    OnAbsence,
    /// Continues after every provider creation failure.
    OnAnyError,
}
