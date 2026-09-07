use thiserror::Error;

/// Errors raised when mutating a provider registry.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[non_exhaustive]
#[must_use]
pub enum RegistryMutationError {
    /// A selector is already owned by another provider.
    #[error("provider selector {selector} claimed by {provider} is already owned by {existing_provider}")]
    DuplicateSelector {
        selector: Box<str>,
        existing_provider: Box<str>,
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

    #[must_use]
    pub fn selector(&self) -> Option<&str> {
        match self {
            Self::DuplicateSelector { selector, .. } => Some(selector),
            Self::Sealed => None,
        }
    }

    #[must_use]
    pub fn existing_provider(&self) -> Option<&str> {
        match self {
            Self::DuplicateSelector { existing_provider, .. } => Some(existing_provider),
            Self::Sealed => None,
        }
    }

    #[must_use]
    pub fn provider(&self) -> Option<&str> {
        match self {
            Self::DuplicateSelector { provider, .. } => Some(provider),
            Self::Sealed => None,
        }
    }

    #[must_use]
    pub const fn is_sealed(&self) -> bool {
        matches!(self, Self::Sealed)
    }
}
