// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while resolving provider selections.

use std::{error::Error, fmt};

use crate::internal::ResolutionErrorRepr;
use crate::{
    AttemptFailure, ProviderSelector, ProviderSelectorError, ResolutionErrorKind,
};

/// Aggregate error returned when provider resolution cannot create a service.
#[derive(Clone, Debug)]
pub struct ResolutionError(ResolutionErrorRepr);

impl ResolutionError {
    /// Creates an error for a direct selector that matched no provider.
    ///
    /// # Arguments
    ///
    /// * `selector` - Valid normalized selector that reached no registry entry.
    ///
    /// # Returns
    ///
    /// An unknown-provider resolution error.
    #[inline]
    #[must_use]
    pub(crate) fn unknown_provider(selector: ProviderSelector) -> Self {
        Self(ResolutionErrorRepr::UnknownProvider { selector })
    }

    /// Creates an error for invalid raw selector input.
    ///
    /// # Arguments
    ///
    /// * `input` - Verbatim selector input supplied by the caller.
    /// * `selector_index` - Zero-based chain position, or `None` for a named
    ///   selector.
    /// * `source` - Selector parsing error that rejected `input`.
    ///
    /// # Returns
    ///
    /// An invalid-selector resolution error retaining its source.
    #[inline]
    #[must_use]
    pub(crate) fn invalid_selector(
        input: &str,
        selector_index: Option<usize>,
        source: ProviderSelectorError,
    ) -> Self {
        Self(ResolutionErrorRepr::InvalidSelector {
            input: input.into(),
            selector_index,
            source,
        })
    }

    /// Creates an error for an empty raw chained selection.
    ///
    /// # Returns
    ///
    /// The empty-selection resolution error.
    #[inline]
    #[must_use]
    pub(crate) const fn empty_selection() -> Self {
        Self(ResolutionErrorRepr::EmptySelection)
    }

    /// Creates an error for automatic resolution from an empty registry.
    ///
    /// # Returns
    ///
    /// The empty-registry resolution error.
    #[inline]
    #[must_use]
    pub(crate) const fn empty_registry() -> Self {
        Self(ResolutionErrorRepr::EmptyRegistry)
    }

    /// Creates an aggregate error when considered candidates produce no service.
    ///
    /// # Arguments
    ///
    /// * `attempts` - Non-empty failures recorded in encounter order.
    ///
    /// # Returns
    ///
    /// An aggregate resolution error owning every failed attempt.
    ///
    /// # Panics
    ///
    /// Panics when the resolver supplies an empty attempt list, which violates
    /// the internal `NoProviderSucceeded` invariant.
    #[inline]
    #[must_use]
    pub(crate) fn no_provider_succeeded(attempts: Vec<AttemptFailure>) -> Self {
        assert!(
            !attempts.is_empty(),
            "no-provider-succeeded errors require at least one attempt",
        );
        Self(ResolutionErrorRepr::NoProviderSucceeded {
            attempts: attempts.into_boxed_slice(),
        })
    }

    /// Returns the overall resolution failure classification.
    ///
    /// # Returns
    ///
    /// The classification corresponding to the retained failure variant.
    #[inline(always)]
    #[must_use]
    pub const fn kind(&self) -> ResolutionErrorKind {
        match self.0 {
            ResolutionErrorRepr::InvalidSelector { .. } => ResolutionErrorKind::InvalidSelector,
            ResolutionErrorRepr::EmptySelection => ResolutionErrorKind::EmptySelection,
            ResolutionErrorRepr::UnknownProvider { .. } => ResolutionErrorKind::UnknownProvider,
            ResolutionErrorRepr::EmptyRegistry => ResolutionErrorKind::EmptyRegistry,
            ResolutionErrorRepr::NoProviderSucceeded { .. } => {
                ResolutionErrorKind::NoProviderSucceeded
            }
        }
    }

    /// Returns selector text retained by a selector-specific failure.
    ///
    /// # Returns
    ///
    /// `Some` with verbatim invalid input or a normalized unknown selector;
    /// returns `None` for failures unrelated to one direct selector.
    #[inline(always)]
    #[must_use]
    pub fn selector_input(&self) -> Option<&str> {
        match &self.0 {
            ResolutionErrorRepr::InvalidSelector { input, .. } => Some(input),
            ResolutionErrorRepr::UnknownProvider { selector } => Some(selector.as_str()),
            ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::EmptyRegistry
            | ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }

    /// Returns the invalid selector's zero-based chain position.
    ///
    /// # Returns
    ///
    /// `Some` for invalid chain input, or `None` for named and non-parse
    /// failures.
    #[inline(always)]
    #[must_use]
    pub const fn selector_index(&self) -> Option<usize> {
        match self.0 {
            ResolutionErrorRepr::InvalidSelector { selector_index, .. } => selector_index,
            ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::UnknownProvider { .. }
            | ResolutionErrorRepr::EmptyRegistry
            | ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }

    /// Returns the selector parsing failure for invalid raw input.
    ///
    /// # Returns
    ///
    /// `Some` for invalid selector syntax, or `None` for every other failure.
    #[inline(always)]
    #[must_use]
    pub fn selector_error(&self) -> Option<&ProviderSelectorError> {
        match &self.0 {
            ResolutionErrorRepr::InvalidSelector { source, .. } => Some(source),
            ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::UnknownProvider { .. }
            | ResolutionErrorRepr::EmptyRegistry
            | ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }

    /// Returns the normalized selector retained for a direct lookup failure.
    ///
    /// # Returns
    ///
    /// `Some` for an unknown direct selector, or `None` for every other
    /// aggregate failure.
    #[inline(always)]
    #[must_use]
    pub fn requested_selector(&self) -> Option<&ProviderSelector> {
        match &self.0 {
            ResolutionErrorRepr::UnknownProvider { selector } => Some(selector),
            ResolutionErrorRepr::InvalidSelector { .. }
            | ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::EmptyRegistry
            | ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }

    /// Returns failed attempts in candidate encounter order.
    ///
    /// # Returns
    ///
    /// The retained attempt slice, or an empty slice for non-aggregate errors.
    #[inline(always)]
    #[must_use]
    pub fn attempts(&self) -> &[AttemptFailure] {
        match &self.0 {
            ResolutionErrorRepr::NoProviderSucceeded { attempts } => attempts,
            ResolutionErrorRepr::InvalidSelector { .. }
            | ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::UnknownProvider { .. }
            | ResolutionErrorRepr::EmptyRegistry => &[],
        }
    }
}

impl fmt::Display for ResolutionError {
    /// Formats the resolution failure and ordered attempt diagnostics.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ResolutionErrorRepr::InvalidSelector {
                input,
                selector_index,
                ..
            } => match selector_index {
                Some(index) => write!(
                    formatter,
                    "invalid provider selector at chain index {index}: {input:?}",
                ),
                None => write!(formatter, "invalid provider selector {input:?}"),
            },
            ResolutionErrorRepr::EmptySelection => {
                formatter.write_str("provider selection chain must not be empty")
            }
            ResolutionErrorRepr::UnknownProvider { selector } => {
                write!(formatter, "unknown provider: {selector}")
            }
            ResolutionErrorRepr::EmptyRegistry => {
                formatter.write_str("cannot resolve a provider from an empty registry")
            }
            ResolutionErrorRepr::NoProviderSucceeded { attempts } => {
                write!(
                    formatter,
                    "no provider succeeded after {} attempt(s)",
                    attempts.len(),
                )?;
                for (index, attempt) in attempts.iter().enumerate() {
                    write!(formatter, "; attempt {}: {attempt}", index + 1)?;
                }
                Ok(())
            }
        }
    }
}

impl Error for ResolutionError {
    /// Returns the selector parsing source for invalid raw input.
    #[inline(always)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.0 {
            ResolutionErrorRepr::InvalidSelector { source, .. } => Some(source),
            ResolutionErrorRepr::EmptySelection
            | ResolutionErrorRepr::UnknownProvider { .. }
            | ResolutionErrorRepr::EmptyRegistry
            | ResolutionErrorRepr::NoProviderSucceeded { .. } => None,
        }
    }
}
