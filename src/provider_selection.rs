// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Validated provider selection inputs.

use crate::error::ProviderSelectionBuildError;
use crate::internal::ProviderSelectionRepr;
use crate::{
    FallbackPolicy,
    ProviderSelector,
};

/// Validated request for the providers a resolver may try.
///
/// Construct selections through [`Self::auto`], [`Self::named`], or
/// [`Self::chain`]. The opaque representation prevents invalid selectors and
/// empty chains from reaching a resolver.
///
/// # Examples
///
/// Parse a configured selection once and reuse it for multiple resolutions:
///
/// ```rust
/// use qubit_spi::{
///     FallbackPolicy,
///     ProviderSelection,
/// };
///
/// let selection = ProviderSelection::chain(["remote", "memory"])?
///     .with_fallback_policy(FallbackPolicy::OnAbsence);
///
/// assert_eq!(2, selection.selectors().len());
/// assert_eq!(FallbackPolicy::OnAbsence, selection.fallback_policy());
/// # Ok::<(), qubit_spi::error::ProviderSelectionBuildError>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSelection {
    /// Invariant-safe target consumed by a provider registry.
    target: ProviderSelectionRepr,
    /// Policy applied after a selected provider fails to create its service.
    fallback_policy: FallbackPolicy,
}

impl ProviderSelection {
    /// Creates an automatic provider selection.
    ///
    /// # Returns
    ///
    /// A selection using deterministic registry priority order.
    #[inline]
    #[must_use]
    pub const fn auto() -> Self {
        Self {
            target: ProviderSelectionRepr::Auto,
            fallback_policy: FallbackPolicy::OnAbsence,
        }
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
    /// Returns [`ProviderSelectionBuildError`] when `value` cannot form a
    /// valid selector.
    #[inline]
    pub fn named(value: &str) -> Result<Self, ProviderSelectionBuildError> {
        let selector = ProviderSelector::parse(value).map_err(|source| {
            ProviderSelectionBuildError::invalid_selector(None, source)
        })?;
        Ok(Self {
            target: ProviderSelectionRepr::Named(selector),
            fallback_policy: FallbackPolicy::OnAbsence,
        })
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
    /// Returns [`ProviderSelectionBuildError`] when any selector is invalid
    /// or when `values` contains no selectors.
    pub fn chain<I, T>(values: I) -> Result<Self, ProviderSelectionBuildError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut selectors = Vec::new();
        for (selector_index, value) in values.into_iter().enumerate() {
            let input = value.as_ref();
            let selector =
                ProviderSelector::parse(input).map_err(|source| {
                    ProviderSelectionBuildError::invalid_selector(
                        Some(selector_index),
                        source,
                    )
                })?;
            selectors.push(selector);
        }
        if selectors.is_empty() {
            return Err(ProviderSelectionBuildError::empty_chain());
        }
        Ok(Self {
            target: ProviderSelectionRepr::Chain(selectors.into_boxed_slice()),
            fallback_policy: FallbackPolicy::OnAbsence,
        })
    }

    /// Returns the explicitly selected provider selectors.
    ///
    /// # Returns
    ///
    /// An empty slice for automatic selection, a one-element slice for named
    /// selection, or the ordered nonempty slice for chained selection.
    #[inline(always)]
    #[must_use]
    pub fn selectors(&self) -> &[ProviderSelector] {
        match &self.target {
            ProviderSelectionRepr::Named(selector) => {
                std::slice::from_ref(selector)
            }
            ProviderSelectionRepr::Chain(selectors) => selectors,
            ProviderSelectionRepr::Auto => &[],
        }
    }

    /// Returns the policy applied after provider creation failures.
    ///
    /// # Returns
    ///
    /// The fallback policy stored with this selection.
    #[inline(always)]
    #[must_use]
    pub const fn fallback_policy(&self) -> FallbackPolicy {
        self.fallback_policy
    }

    /// Replaces the policy applied after provider creation failures.
    ///
    /// # Arguments
    ///
    /// * `fallback_policy` - Replacement policy for candidate traversal.
    ///
    /// # Returns
    ///
    /// This selection with its target unchanged and policy replaced.
    #[inline(always)]
    #[must_use]
    pub const fn with_fallback_policy(
        mut self,
        fallback_policy: FallbackPolicy,
    ) -> Self {
        self.fallback_policy = fallback_policy;
        self
    }

    /// Returns the validated representation consumed by the resolver.
    ///
    /// # Returns
    ///
    /// A shared reference to the invariant-safe private representation.
    #[inline(always)]
    pub(crate) const fn repr(&self) -> &ProviderSelectionRepr {
        &self.target
    }
}

impl Default for ProviderSelection {
    /// Creates the default automatic provider selection.
    ///
    /// # Returns
    ///
    /// An automatic provider selection.
    #[inline(always)]
    fn default() -> Self {
        Self::auto()
    }
}
