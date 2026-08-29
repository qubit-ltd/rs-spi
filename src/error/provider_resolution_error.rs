// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while resolving provider selections.

use std::error::Error;
use std::fmt;

use crate::ProviderSelector;

/// Error returned when a Registry cannot resolve provider candidates.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderResolutionError {
    /// One or more required selectors matched no registered provider.
    #[non_exhaustive]
    UnknownProviders {
        /// Unknown normalized selectors retained in input order.
        selectors: Box<[ProviderSelector]>,
    },
    /// A nonempty selector chain matched no registered provider candidates.
    #[non_exhaustive]
    NoCandidates {
        /// Normalized selectors that matched no candidates, in input order.
        selectors: Box<[ProviderSelector]>,
    },
    /// Automatic selection was requested from an empty Registry.
    EmptyRegistry,
}

impl ProviderResolutionError {
    /// Returns the selectors retained by this resolution failure.
    ///
    /// # Returns
    ///
    /// The selectors for [`Self::UnknownProviders`] or
    /// [`Self::NoCandidates`], or `None` for [`Self::EmptyRegistry`].
    #[inline(always)]
    #[must_use]
    pub fn selectors(&self) -> Option<&[ProviderSelector]> {
        match self {
            Self::UnknownProviders { selectors } | Self::NoCandidates { selectors } => Some(selectors),
            Self::EmptyRegistry => None,
        }
    }

    /// Tests whether required selectors matched no registered provider.
    ///
    /// # Returns
    ///
    /// `true` for [`Self::UnknownProviders`]; otherwise `false`.
    #[inline(always)]
    #[must_use]
    pub const fn is_unknown_providers(&self) -> bool {
        matches!(self, Self::UnknownProviders { .. })
    }

    /// Tests whether a selector chain yielded no provider candidates.
    ///
    /// # Returns
    ///
    /// `true` for [`Self::NoCandidates`]; otherwise `false`.
    #[inline(always)]
    #[must_use]
    pub const fn is_no_candidates(&self) -> bool {
        matches!(self, Self::NoCandidates { .. })
    }

    /// Tests whether automatic selection targeted an empty Registry.
    ///
    /// # Returns
    ///
    /// `true` for [`Self::EmptyRegistry`]; otherwise `false`.
    #[inline(always)]
    #[must_use]
    pub const fn is_empty_registry(&self) -> bool {
        matches!(self, Self::EmptyRegistry)
    }

    /// Creates an error for required selectors that matched no provider.
    ///
    /// # Parameters
    ///
    /// * `selectors` - Nonempty unknown selectors in input order.
    ///
    /// # Returns
    ///
    /// An unknown-providers resolution error.
    ///
    /// # Panics
    ///
    /// Panics when `selectors` is empty.
    #[inline]
    #[must_use]
    pub(crate) fn unknown_providers(selectors: Vec<ProviderSelector>) -> Self {
        assert!(
            !selectors.is_empty(),
            "unknown-provider errors require at least one selector",
        );
        Self::UnknownProviders {
            selectors: selectors.into_boxed_slice(),
        }
    }

    /// Creates an error when a selector chain yields no provider candidates.
    ///
    /// # Parameters
    ///
    /// * `selectors` - Non-empty normalized selectors in input order.
    ///
    /// # Returns
    ///
    /// A no-candidates resolution error retaining every requested selector.
    ///
    /// # Panics
    ///
    /// Panics when `selectors` is empty.
    #[inline]
    #[must_use]
    pub(crate) fn no_candidates(selectors: Vec<ProviderSelector>) -> Self {
        assert!(
            !selectors.is_empty(),
            "no-candidates errors require at least one selector",
        );
        Self::NoCandidates {
            selectors: selectors.into_boxed_slice(),
        }
    }

    /// Creates an error for automatic selection from an empty Registry.
    ///
    /// # Returns
    ///
    /// The empty-Registry resolution error.
    #[inline]
    #[must_use]
    pub(crate) const fn empty_registry() -> Self {
        Self::EmptyRegistry
    }
}

impl fmt::Display for ProviderResolutionError {
    /// Formats the provider resolution failure.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination formatter.
    ///
    /// # Returns
    ///
    /// The formatter result.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] when the formatter rejects diagnostic output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProviders { selectors } => {
                if selectors.len() == 1 {
                    formatter.write_str("unknown provider selector")?;
                } else {
                    formatter.write_str("unknown provider selectors")?;
                }
                for selector in selectors {
                    write!(formatter, "; {selector}")?;
                }
                Ok(())
            }
            Self::NoCandidates { selectors } => {
                formatter.write_str("no provider candidates matched")?;
                for selector in selectors {
                    write!(formatter, "; {selector}")?;
                }
                Ok(())
            }
            Self::EmptyRegistry => formatter.write_str("cannot select a provider from an empty registry"),
        }
    }
}

impl Error for ProviderResolutionError {}
