// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy-driven provider selection and service creation.

use std::{
    collections::HashSet,
    fmt,
};

use crate::error::{
    AttemptFailure,
    ProviderErrorKind,
    ResolutionError,
};
use crate::internal::ProviderSelectionRepr;
use crate::{
    CreatedService,
    FallbackPolicy,
    ProviderRegistry,
    ProviderSelection,
    ProviderSelector,
    ServiceSpec,
};

/// Creates services from an immutable registry using explicit fallback policy.
///
/// Applications construct a resolver after startup and use it at runtime to
/// apply per-call [`ProviderSelection`] values to a typed service registry.
pub struct ProviderResolver<S>
where
    S: ServiceSpec,
{
    /// Immutable catalog searched for provider candidates.
    registry: ProviderRegistry<S>,
    /// Policy deciding which provider failures permit fallback.
    fallback_policy: FallbackPolicy,
}

impl<S> ProviderResolver<S>
where
    S: ServiceSpec,
{
    /// Creates a resolver over an immutable provider registry.
    ///
    /// # Arguments
    ///
    /// * `registry` - Immutable catalog of selectable providers.
    /// * `fallback_policy` - Policy controlling which failures may continue.
    ///
    /// # Returns
    ///
    /// A resolver combining the catalog and fallback policy.
    #[inline]
    #[must_use]
    pub fn new(
        registry: ProviderRegistry<S>,
        fallback_policy: FallbackPolicy,
    ) -> Self {
        Self {
            registry,
            fallback_policy,
        }
    }

    /// Returns the immutable provider registry used by this resolver.
    ///
    /// # Returns
    ///
    /// The immutable provider catalog.
    #[inline(always)]
    #[must_use]
    pub const fn registry(&self) -> &ProviderRegistry<S> {
        &self.registry
    }

    /// Returns the fallback policy applied after provider creation failures.
    ///
    /// # Returns
    ///
    /// The policy controlling subsequent provider attempts.
    #[inline(always)]
    #[must_use]
    pub const fn fallback_policy(&self) -> FallbackPolicy {
        self.fallback_policy
    }

    /// Creates a service using the requested provider selection.
    ///
    /// # Arguments
    ///
    /// * `selection` - Validated candidate selection and ordering.
    /// * `config` - Configuration passed unchanged to attempted providers.
    ///
    /// # Returns
    ///
    /// The created service and canonical winning provider ID.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when a named selector is unknown, no
    /// candidate succeeds, or a failure is disallowed by the fallback policy.
    #[inline]
    pub fn create(
        &self,
        selection: &ProviderSelection,
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError> {
        match selection.repr() {
            ProviderSelectionRepr::Auto => self.create_automatic(config),
            ProviderSelectionRepr::Named(selector) => {
                self.create_named_selector(selector, config)
            }
            ProviderSelectionRepr::Chain(selectors) => {
                self.create_selector_chain(selectors, config)
            }
        }
    }

    /// Creates a service using deterministic automatic provider order.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration forwarded to each provider candidate.
    ///
    /// # Returns
    ///
    /// The first successfully created service and canonical provider ID.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when the registry is empty, no provider
    /// succeeds, or fallback policy stops after a provider failure.
    #[inline(always)]
    pub fn create_auto(
        &self,
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError> {
        self.create_automatic(config)
    }

    /// Creates a service through one raw provider selector.
    ///
    /// # Arguments
    ///
    /// * `selector` - Raw selector normalized and validated before lookup.
    /// * `config` - Configuration forwarded only to the matching provider.
    ///
    /// # Returns
    ///
    /// The selected provider's service and canonical provider ID.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when `selector` is invalid or unknown, or
    /// when the selected provider cannot create its service.
    ///
    /// # Performance
    ///
    /// TODO: Before adding a no-allocation fast path, benchmark representative
    /// repeated canonical-selector lookups, including filesystem URI schemes,
    /// and retain the optimization only when the measurements show a material
    /// benefit.
    #[inline]
    pub fn create_named(
        &self,
        selector: impl AsRef<str>,
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError> {
        let input = selector.as_ref();
        let selection =
            ProviderSelection::named(input).map_err(|error| match error {
                crate::error::ProviderSelectionError::InvalidSelector {
                    source,
                    ..
                } => ResolutionError::invalid_selector(input, None, source),
                crate::error::ProviderSelectionError::EmptyChain => {
                    ResolutionError::empty_selection()
                }
            })?;
        self.create(&selection, config)
    }

    /// Creates a service through a nonempty sequence of raw selectors.
    ///
    /// Each selector is normalized and validated before any provider is tried.
    /// Providers are attempted in input order and aliases resolving to an
    /// already attempted provider are skipped.
    ///
    /// # Arguments
    ///
    /// * `selectors` - Raw selectors in desired attempt order.
    /// * `config` - Configuration forwarded to each distinct provider.
    ///
    /// # Returns
    ///
    /// The first successfully created service and canonical provider ID.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when the chain is empty, one selector is
    /// invalid, no provider succeeds, or fallback policy stops resolution.
    pub fn create_chain<I, T>(
        &self,
        selectors: I,
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let selection = ProviderSelection::chain(selectors)
            .map_err(ResolutionError::from)?;
        self.create(&selection, config)
    }

    /// Creates a service by trying providers in automatic order.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration forwarded to each candidate.
    ///
    /// # Returns
    ///
    /// The first successful service and canonical provider ID.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when the registry is empty, every attempted
    /// provider fails, or the fallback policy stops resolution after a provider
    /// failure. The error contains failures recorded before resolution stopped.
    fn create_automatic(
        &self,
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError> {
        if self.registry.is_empty() {
            return Err(ResolutionError::empty_registry());
        }
        let mut failures = Vec::new();
        for &index in self.registry.automatic_indices() {
            let resolved = self.registry.resolved_at(index);
            match resolved.create(config) {
                Ok(service) => {
                    let provider_id = resolved.descriptor().id().clone();
                    return Ok(CreatedService::new(provider_id, service));
                }
                Err(error) => {
                    let provider_id = resolved.descriptor().id().clone();
                    let error_kind = error.kind();
                    failures.push(AttemptFailure::from_provider_error(
                        None,
                        provider_id,
                        error,
                    ));
                    if !self.should_continue(error_kind) {
                        return Err(ResolutionError::stopped_by_policy(
                            failures,
                        ));
                    }
                }
            }
        }
        Err(ResolutionError::exhausted(failures))
    }

    /// Creates a service through one explicitly selected provider.
    ///
    /// Named selections never fall back.
    ///
    /// # Arguments
    ///
    /// * `selector` - Valid normalized selector used for lookup.
    /// * `config` - Configuration forwarded to the matching factory.
    ///
    /// # Returns
    ///
    /// The selected provider's service and canonical provider ID.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when lookup is unknown or provider creation
    /// fails.
    #[inline]
    fn create_named_selector(
        &self,
        selector: &ProviderSelector,
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError> {
        let Some(index) = self.registry.index_for(selector) else {
            return Err(ResolutionError::unknown_provider(selector.clone()));
        };
        let resolved = self.registry.resolved_at(index);
        let provider_id = resolved.descriptor().id().clone();
        match resolved.create(config) {
            Ok(service) => Ok(CreatedService::new(provider_id, service)),
            Err(error) => Err(ResolutionError::exhausted(vec![
                AttemptFailure::from_provider_error(
                    Some(selector.clone()),
                    provider_id,
                    error,
                ),
            ])),
        }
    }

    /// Creates a service by trying the supplied selectors in order.
    ///
    /// `selectors` may contain aliases for the same provider, which is tried
    /// only once; later selectors resolving to that provider are omitted
    /// without a failure record.
    ///
    /// # Arguments
    ///
    /// * `selectors` - Valid normalized selectors in attempt order.
    /// * `config` - Configuration forwarded to each distinct provider.
    ///
    /// # Returns
    ///
    /// The first successfully created service and canonical provider ID.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when no provider succeeds or the fallback
    /// policy stops the chain. The error records unknown selectors and actual
    /// provider failures encountered before resolution stopped.
    fn create_selector_chain(
        &self,
        selectors: &[ProviderSelector],
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError> {
        let mut attempted = HashSet::new();
        let mut failures = Vec::new();
        for selector in selectors {
            let Some(index) = self.registry.index_for(selector) else {
                failures
                    .push(AttemptFailure::unknown_provider(selector.clone()));
                continue;
            };
            if !attempted.insert(index) {
                continue;
            }
            let resolved = self.registry.resolved_at(index);
            let provider_id = resolved.descriptor().id().clone();
            match resolved.create(config) {
                Ok(service) => {
                    return Ok(CreatedService::new(provider_id, service));
                }
                Err(error) => {
                    let error_kind = error.kind();
                    failures.push(AttemptFailure::from_provider_error(
                        Some(selector.clone()),
                        provider_id,
                        error,
                    ));
                    if !self.should_continue(error_kind) {
                        return Err(ResolutionError::stopped_by_policy(
                            failures,
                        ));
                    }
                }
            }
        }
        Err(ResolutionError::exhausted(failures))
    }

    /// Determines whether this resolver may fall back after an error kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - Provider-reported failure classification.
    ///
    /// # Returns
    ///
    /// `true` when the resolver's policy permits another attempt.
    #[inline(always)]
    fn should_continue(&self, kind: ProviderErrorKind) -> bool {
        match self.fallback_policy {
            FallbackPolicy::OnAnyError => true,
            FallbackPolicy::OnAbsence => {
                matches!(
                    kind,
                    ProviderErrorKind::Unsupported
                        | ProviderErrorKind::Unavailable
                )
            }
        }
    }
}

impl<S> Clone for ProviderResolver<S>
where
    S: ServiceSpec,
{
    /// Clones the resolver and its shared immutable registry handle.
    ///
    /// # Returns
    ///
    /// A resolver with the same registry and fallback policy.
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            fallback_policy: self.fallback_policy,
        }
    }
}

impl<S> fmt::Debug for ProviderResolver<S>
where
    S: ServiceSpec,
{
    /// Formats the registry metadata and fallback policy.
    ///
    /// # Arguments
    ///
    /// * `formatter` - Destination formatter.
    ///
    /// # Returns
    ///
    /// The formatter result.
    ///
    /// # Errors
    ///
    /// Returns [`fmt::Error`] when the formatter rejects debug output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderResolver")
            .field("registry", &self.registry)
            .field("fallback_policy", &self.fallback_policy)
            .finish()
    }
}
