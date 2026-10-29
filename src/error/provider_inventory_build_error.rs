// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors raised while building a provider registry from link-time discovery.

use thiserror::Error;

use crate::error::RegistryMutationError;
use crate::inventory::ProviderRegistrationSource;

/// Error raised when a discovered provider cannot be added to a registry.
///
/// This error is produced only by a service family's generated
/// `build_registry()` function. Applications should handle it at startup using
/// the source and optional registration-error accessors; they do not construct
/// it directly.
///
/// # Examples
///
/// ```rust
/// mod example {
/// use std::convert::Infallible;
///
/// use qubit_spi::error::ProviderFailure;
/// use qubit_spi::error::RegistryMutationError;
/// use qubit_spi::{
///     ProviderDescriptor, ProviderMetadata, ServiceProvider, ServiceSpec, SyncServiceSpec,
/// };
///
/// struct Spec;
/// impl ServiceSpec for Spec {
///     type Config = ();
///     type Error = Infallible;
/// }
/// impl SyncServiceSpec for Spec { type Output = (); }
///
/// struct First;
/// impl ProviderMetadata for First {
///     fn descriptor(&self) -> ProviderDescriptor {
///         qubit_spi::provider_descriptor!("duplicate")
///     }
/// }
/// impl ServiceProvider<Spec> for First {
///     fn create_configured(&self, _: &()) -> Result<(), ProviderFailure<Infallible>> {
///         Ok(())
///     }
/// }
///
/// struct Second;
/// impl ProviderMetadata for Second {
///     fn descriptor(&self) -> ProviderDescriptor {
///         qubit_spi::provider_descriptor!("duplicate")
///     }
/// }
/// impl ServiceProvider<Spec> for Second {
///     fn create_configured(&self, _: &()) -> Result<(), ProviderFailure<Infallible>> {
///         Ok(())
///     }
/// }
///
/// qubit_spi::declare_sync_provider_inventory! {
///     pub mod providers {
///         spec = Spec;
///     }
/// }
/// qubit_spi::submit_sync_provider! {
///     inventory_entry = providers::Entry;
///     spec = Spec;
///     provider = First;
/// }
/// qubit_spi::submit_sync_provider! {
///     inventory_entry = providers::Entry;
///     spec = Spec;
///     provider = Second;
/// }
///
/// qubit_spi::declare_sync_provider_inventory! {
///     pub mod ambiguous_providers {
///         spec = Spec;
///     }
/// }
///
/// macro_rules! submit_ambiguous_providers {
///     () => {
///         qubit_spi::submit_sync_provider! {
///             inventory_entry = ambiguous_providers::Entry;
///             spec = Spec;
///             provider = First;
///         }
///         qubit_spi::submit_sync_provider! {
///             inventory_entry = ambiguous_providers::Entry;
///             spec = Spec;
///             provider = Second;
///         }
///     };
/// }
/// submit_ambiguous_providers!();
///
/// pub fn run() {
///     let error = providers::build_registry().expect_err("duplicate IDs conflict");
///     assert!(error.source_location().line() > 0);
///     assert!(matches!(error.registration_error(), Some(RegistryMutationError::DuplicateSelector { .. })));
///
///     let error = ambiguous_providers::build_registry().expect_err("macro submissions share a source");
///     assert!(error.source_location().line() > 0);
///     assert!(error.registration_error().is_none());
/// }
/// }
/// example::run();
/// ```
#[derive(Clone, Debug, Error)]
#[non_exhaustive]
#[must_use]
pub enum ProviderInventoryBuildError {
    /// Multiple discovered entries use the same declaration source.
    #[error("provider registration source {location} is shared by multiple inventory entries; no factories were run")]
    AmbiguousSource {
        /// Declaration location shared by the ambiguous submissions.
        location: ProviderRegistrationSource,
    },
    /// A submitted provider conflicts with registry mutation rules.
    #[error("failed to register provider submitted from {source}: {error}")]
    Registration {
        /// Declaration location of the submitted provider.
        source: ProviderRegistrationSource,
        /// Registry mutation failure that rejected the provider.
        #[source]
        error: RegistryMutationError,
    },
}

impl ProviderInventoryBuildError {
    /// Creates an error for multiple entries with the same declaration source.
    ///
    /// # Parameters
    ///
    /// * `source` - Declaration location shared by the discovered entries.
    ///
    /// # Returns
    ///
    /// An ambiguity error identifying the source whose factories were skipped.
    pub(crate) fn ambiguous_source(source: ProviderRegistrationSource) -> Self {
        Self::AmbiguousSource { location: source }
    }

    /// Creates an error for a discovered provider rejected during registration.
    ///
    /// # Parameters
    ///
    /// * `source` - Declaration location of the rejected provider.
    /// * `error` - Registry mutation failure that rejected the provider.
    ///
    /// # Returns
    ///
    /// A registration error retaining both the discovery source and mutation
    /// failure.
    #[allow(dead_code)] // Used only when inventory support is enabled.
    pub(crate) fn registration(source: ProviderRegistrationSource, error: RegistryMutationError) -> Self {
        Self::Registration { source, error }
    }

    /// Returns the declaration source associated with this build error.
    ///
    /// # Returns
    ///
    /// The source location retained for diagnostics.
    #[must_use]
    #[inline]
    pub const fn source_location(&self) -> ProviderRegistrationSource {
        match self {
            Self::AmbiguousSource { location } => *location,
            Self::Registration { source, .. } => *source,
        }
    }

    /// Returns the registry mutation error when registration was attempted.
    ///
    /// # Returns
    ///
    /// The underlying registration failure, or `None` when duplicate sources
    /// prevented factories from running.
    #[must_use = "check whether registry mutation caused this inventory build error"]
    #[inline]
    pub const fn registration_error(&self) -> Option<&RegistryMutationError> {
        match self {
            Self::AmbiguousSource { .. } => None,
            Self::Registration { error, .. } => Some(error),
        }
    }
}
