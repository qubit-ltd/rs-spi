// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Validated provider selection inputs.

use crate::FallbackPolicy;
use crate::MissingProviderPolicy;
use crate::ProviderSelectionTargetRef;
use crate::ProviderSelector;
use crate::error::ProviderSelectionBuildError;
use crate::selection::ProviderSelectionRepr;

/// Validated request for the providers a resolver may try.
///
/// Construct selections through [`Self::auto`], [`Self::named`],
/// [`Self::chain`], or [`Self::chain_allowing_missing`]. The opaque
/// representation prevents invalid selectors and empty chains from reaching a
/// resolver.
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
/// assert!(matches!(
///     selection.target(),
///     qubit_spi::ProviderSelectionTargetRef::Chain { selectors, .. }
///         if selectors.len() == 2
/// ));
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
    /// # Parameters
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
        let selector = ProviderSelector::parse(value)
            .map_err(|source| ProviderSelectionBuildError::invalid_selector(None, source))?;
        Ok(Self {
            target: ProviderSelectionRepr::Named(selector),
            fallback_policy: FallbackPolicy::OnAbsence,
        })
    }

    /// Creates a nonempty ordered candidate chain from configuration input.
    ///
    /// # Type Parameters
    ///
    /// * `I` - Iterator-like source of selector inputs.
    /// * `T` - Individual selector input convertible to a string reference.
    ///
    /// # Parameters
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
    #[inline(always)]
    pub fn chain<I, T>(values: I) -> Result<Self, ProviderSelectionBuildError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        Self::build_chain(values, MissingProviderPolicy::Reject)
    }

    /// Creates a nonempty ordered chain that ignores unknown selectors.
    ///
    /// # Type Parameters
    ///
    /// * `I` - Iterator-like source of selector inputs.
    /// * `T` - Individual selector input convertible to a string reference.
    ///
    /// # Parameters
    ///
    /// * `values` - Raw selectors normalized in encounter order.
    ///
    /// # Returns
    ///
    /// A validated nonempty selector chain that explicitly permits missing
    /// provider registrations.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderSelectionBuildError`] when any selector is invalid
    /// or when `values` contains no selectors.
    #[inline(always)]
    pub fn chain_allowing_missing<I, T>(values: I) -> Result<Self, ProviderSelectionBuildError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        Self::build_chain(values, MissingProviderPolicy::Ignore)
    }

    /// Builds a validated chain with the specified missing-provider policy.
    ///
    /// # Type Parameters
    ///
    /// * `I` - Iterator-like source of selector inputs.
    /// * `T` - Individual selector input convertible to a string reference.
    ///
    /// # Parameters
    ///
    /// * `values` - Raw selectors normalized in encounter order.
    /// * `missing_policy` - Policy retained with the chain target.
    ///
    /// # Returns
    ///
    /// A validated nonempty selector chain.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderSelectionBuildError`] when any selector is invalid
    /// or when `values` contains no selectors.
    fn build_chain<I, T>(values: I, missing_policy: MissingProviderPolicy) -> Result<Self, ProviderSelectionBuildError>
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
        Ok(Self {
            target: ProviderSelectionRepr::Chain {
                selectors: selectors.into_boxed_slice(),
                missing_policy,
            },
            fallback_policy: FallbackPolicy::OnAbsence,
        })
    }

    /// Returns a lossless borrowed view of the selection target.
    ///
    /// # Returns
    ///
    /// A view distinguishing automatic, named, strict-chain, and lenient-chain
    /// targets without allocating.
    #[inline(always)]
    #[must_use]
    pub const fn target(&self) -> ProviderSelectionTargetRef<'_> {
        match &self.target {
            ProviderSelectionRepr::Auto => ProviderSelectionTargetRef::Auto,
            ProviderSelectionRepr::Named(selector) => ProviderSelectionTargetRef::Named(selector),
            ProviderSelectionRepr::Chain {
                selectors,
                missing_policy,
            } => ProviderSelectionTargetRef::Chain {
                selectors,
                missing_policy: *missing_policy,
            },
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
    /// # Parameters
    ///
    /// * `fallback_policy` - Replacement policy for candidate traversal.
    ///
    /// # Returns
    ///
    /// This selection with its target unchanged and policy replaced.
    #[inline(always)]
    #[must_use]
    pub const fn with_fallback_policy(mut self, fallback_policy: FallbackPolicy) -> Self {
        self.fallback_policy = fallback_policy;
        self
    }

    /// Returns the validated representation consumed by the resolver.
    ///
    /// # Returns
    ///
    /// A shared reference to the invariant-safe private representation.
    #[inline(always)]
    #[must_use]
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
