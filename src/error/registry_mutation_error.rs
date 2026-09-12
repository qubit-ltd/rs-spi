use thiserror::Error;

/// Errors raised when mutating a provider registry.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
#[must_use]
pub enum RegistryMutationError {
    /// A selector is already owned by another provider.
    #[error("provider selector {selector} claimed by {provider} is already owned by {existing_provider}")]
    DuplicateSelector {
        /// Selector that has already been claimed.
        selector: Box<str>,
        /// Identifier of the provider that already owns the selector.
        existing_provider: Box<str>,
        /// Identifier of the provider that attempted to claim the selector.
        provider: Box<str>,
    },
    /// The registry has been sealed and cannot be mutated.
    #[error("provider registry is sealed")]
    Sealed,
}

impl RegistryMutationError {
    pub(crate) fn duplicate_selector(selector: &str, existing_provider: &str, provider: &str) -> Self {
        Self::DuplicateSelector {
            selector: selector.into(),
            existing_provider: existing_provider.into(),
            provider: provider.into(),
        }
    }

    /// Returns the duplicate selector, or [`None`] when the registry is sealed.
    #[must_use]
    pub fn selector(&self) -> Option<&str> {
        match self {
            Self::DuplicateSelector { selector, .. } => Some(selector),
            Self::Sealed => None,
        }
    }

    /// Returns the provider that already owns the selector, or [`None`] when
    /// the registry is sealed.
    #[must_use]
    pub fn existing_provider(&self) -> Option<&str> {
        match self {
            Self::DuplicateSelector { existing_provider, .. } => Some(existing_provider),
            Self::Sealed => None,
        }
    }

    /// Returns the provider that attempted to claim the selector, or [`None`]
    /// when the registry is sealed.
    #[must_use]
    pub fn provider(&self) -> Option<&str> {
        match self {
            Self::DuplicateSelector { provider, .. } => Some(provider),
            Self::Sealed => None,
        }
    }

    /// Returns `true` when the registry has been sealed and rejects mutations.
    #[must_use]
    pub const fn is_sealed(&self) -> bool {
        matches!(self, Self::Sealed)
    }
}
