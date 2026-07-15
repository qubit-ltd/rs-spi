// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provider-level service creation errors.

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::Arc;

/// Error returned by one provider while creating a service.
#[derive(Debug, Clone)]
pub enum ProviderCreateError {
    /// The provider discovered at creation time that it cannot be used.
    Unavailable {
        /// Human-readable unavailability reason.
        reason: String,
        /// Lower-level cause, when the provider can expose one.
        source: Option<Arc<dyn Error + Send + Sync + 'static>>,
    },
    /// The provider failed while creating the service.
    Failed {
        /// Human-readable failure reason.
        reason: String,
        /// Lower-level cause, when the provider can expose one.
        source: Option<Arc<dyn Error + Send + Sync + 'static>>,
    },
}

impl ProviderCreateError {
    /// Creates an unavailable-provider error.
    ///
    /// # Parameters
    /// - `reason`: Human-readable unavailability reason.
    ///
    /// # Returns
    /// Provider creation error.
    #[inline]
    pub fn unavailable(reason: &str) -> Self {
        Self::Unavailable {
            reason: reason.to_owned(),
            source: None,
        }
    }

    /// Creates an unavailable-provider error with a source error.
    ///
    /// # Parameters
    /// - `reason`: Human-readable unavailability reason.
    /// - `source`: Lower-level cause that made the provider unavailable.
    ///
    /// # Returns
    /// Provider creation error preserving the supplied source.
    #[inline]
    pub fn unavailable_with_source<E>(reason: &str, source: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::Unavailable {
            reason: reason.to_owned(),
            source: Some(Arc::new(source)),
        }
    }

    /// Creates a provider failure error.
    ///
    /// # Parameters
    /// - `reason`: Human-readable failure reason.
    ///
    /// # Returns
    /// Provider creation error.
    #[inline]
    pub fn failed(reason: &str) -> Self {
        Self::Failed {
            reason: reason.to_owned(),
            source: None,
        }
    }

    /// Creates a provider failure error with a source error.
    ///
    /// # Parameters
    /// - `reason`: Human-readable failure reason.
    /// - `source`: Lower-level cause that made service creation fail.
    ///
    /// # Returns
    /// Provider creation error preserving the supplied source.
    #[inline]
    pub fn failed_with_source<E>(reason: &str, source: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self::Failed {
            reason: reason.to_owned(),
            source: Some(Arc::new(source)),
        }
    }

    /// Gets the provider-level reason.
    ///
    /// # Returns
    /// Human-readable reason carried by this error.
    #[inline]
    pub fn reason(&self) -> &str {
        match self {
            Self::Unavailable { reason, .. } | Self::Failed { reason, .. } => reason,
        }
    }

    /// Tells whether this error represents provider unavailability.
    ///
    /// # Returns
    /// `true` when the provider reported itself unavailable.
    #[inline]
    pub(crate) fn is_unavailable(&self) -> bool {
        matches!(self, Self::Unavailable { .. })
    }

    /// Gets the lower-level source error.
    ///
    /// # Returns
    /// `Some` source error when one was supplied, or `None` otherwise.
    #[inline]
    fn source_error(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Unavailable { source, .. } | Self::Failed { source, .. } => source
                .as_deref()
                .map(|source| source as &(dyn Error + 'static)),
        }
    }
}

impl Display for ProviderCreateError {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Unavailable { reason, .. } => {
                write!(formatter, "provider is unavailable: {reason}")
            }
            Self::Failed { reason, .. } => {
                write!(formatter, "provider failed to create service: {reason}")
            }
        }
    }
}

impl Error for ProviderCreateError {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source_error()
    }
}
