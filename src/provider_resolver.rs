// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy-driven provider selection and service creation.

use std::{collections::HashSet, fmt};

use crate::provider_selection::ProviderSelectionRepr;
use crate::{
    AttemptFailure, CreatedService, FallbackPolicy, ProviderErrorKind, ProviderRegistry,
    ProviderSelection, ProviderSelector, ResolutionError, ServiceSpec,
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
    /// `registry` supplies the selectable providers and `fallback_policy`
    /// controls which failed attempts may continue. Returns the resolver.
    #[must_use]
    pub fn new(registry: ProviderRegistry<S>, fallback_policy: FallbackPolicy) -> Self {
        Self {
            registry,
            fallback_policy,
        }
    }

    /// Returns the immutable provider registry used by this resolver.
    #[must_use]
    pub const fn registry(&self) -> &ProviderRegistry<S> {
        &self.registry
    }

    /// Returns the fallback policy applied after provider creation failures.
    #[must_use]
    pub const fn fallback_policy(&self) -> FallbackPolicy {
        self.fallback_policy
    }

    /// Creates a service using the requested provider selection.
    ///
    /// `selection` controls candidate ordering and `config` is passed unchanged
    /// to each attempted provider. Returns the service and its winning provider
    /// identity on success.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when a named selector is unknown, no
    /// candidate succeeds, or a failure is disallowed by the fallback policy.
    pub fn create(
        &self,
        selection: &ProviderSelection,
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError> {
        match selection.repr() {
            ProviderSelectionRepr::Auto => self.create_automatic(config),
            ProviderSelectionRepr::Named(selector) => self.create_named_selector(selector, config),
            ProviderSelectionRepr::Chain(selectors) => {
                self.create_selector_chain(selectors, config)
            }
        }
    }

    /// Creates a service using deterministic automatic provider order.
    ///
    /// `config` is forwarded to each provider candidate. Returns the first
    /// successfully created service and its canonical provider ID.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when the registry is empty, no provider
    /// succeeds, or fallback policy stops after a provider failure.
    pub fn create_auto(
        &self,
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError> {
        self.create_automatic(config)
    }

    /// Creates a service through one raw provider selector.
    ///
    /// `selector` is normalized and validated before lookup. `config` is
    /// forwarded only to the matching provider.
    ///
    /// # Errors
    ///
    /// Returns [`ResolutionError`] when `selector` is invalid or unknown, or
    /// when the selected provider cannot create its service.
    pub fn create_named(
        &self,
        selector: impl AsRef<str>,
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError> {
        let input = selector.as_ref();
        let selector = ProviderSelector::parse(input)
            .map_err(|source| ResolutionError::invalid_selector(input, None, source))?;
        self.create_named_selector(&selector, config)
    }

    /// Creates a service through a nonempty sequence of raw selectors.
    ///
    /// Each selector is normalized and validated before any provider is tried.
    /// Providers are attempted in input order and aliases resolving to an
    /// already attempted provider are skipped.
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
        let mut parsed = Vec::new();
        for (selector_index, selector) in selectors.into_iter().enumerate() {
            let input = selector.as_ref();
            let selector = ProviderSelector::parse(input).map_err(|source| {
                ResolutionError::invalid_selector(input, Some(selector_index), source)
            })?;
            parsed.push(selector);
        }
        if parsed.is_empty() {
            return Err(ResolutionError::empty_selection());
        }
        self.create_selector_chain(&parsed, config)
    }

    /// Creates a service by trying providers in automatic order.
    ///
    /// `config` is forwarded to each candidate. Returns the first successful
    /// service and stops without invoking later candidates.
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
                    failures.push(AttemptFailure::provider_error(None, provider_id, &error));
                    if !self.should_continue(error.kind()) {
                        return Err(ResolutionError::no_provider_succeeded(failures));
                    }
                }
            }
        }
        Err(ResolutionError::no_provider_succeeded(failures))
    }

    /// Creates a service through one explicitly selected provider.
    ///
    /// `selector` is normalized and `config` is forwarded to the matching
    /// factory. Returns its service, or a [`ResolutionError`] if it is unknown
    /// or creation fails; named selections never fall back.
    fn create_named_selector(
        &self,
        selector: &ProviderSelector,
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError> {
        let Some(index) = self.registry.index_for(selector) else {
            return Err(ResolutionError::unknown_provider(selector.as_str()));
        };
        let resolved = self.registry.resolved_at(index);
        let provider_id = resolved.descriptor().id().clone();
        match resolved.create(config) {
            Ok(service) => Ok(CreatedService::new(provider_id, service)),
            Err(error) => Err(ResolutionError::no_provider_succeeded(vec![
                AttemptFailure::provider_error(Some(selector.clone()), provider_id, &error),
            ])),
        }
    }

    /// Creates a service by trying the supplied selectors in order.
    ///
    /// `selectors` may contain aliases for the same provider, which is tried
    /// only once; later selectors resolving to that provider are omitted without
    /// a failure record. `config` is forwarded to each attempted factory. Returns
    /// the first successful service.
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
                failures.push(AttemptFailure::unknown_provider(selector.clone()));
                continue;
            };
            if !attempted.insert(index) {
                continue;
            }
            let resolved = self.registry.resolved_at(index);
            let provider_id = resolved.descriptor().id().clone();
            match resolved.create(config) {
                Ok(service) => return Ok(CreatedService::new(provider_id, service)),
                Err(error) => {
                    failures.push(AttemptFailure::provider_error(
                        Some(selector.clone()),
                        provider_id,
                        &error,
                    ));
                    if !self.should_continue(error.kind()) {
                        return Err(ResolutionError::no_provider_succeeded(failures));
                    }
                }
            }
        }
        Err(ResolutionError::no_provider_succeeded(failures))
    }

    /// Determines whether this resolver may fall back after an error kind.
    ///
    /// `kind` is the provider-reported failure classification. Returns `true`
    /// exactly when this resolver's fallback policy permits another attempt.
    fn should_continue(&self, kind: ProviderErrorKind) -> bool {
        match self.fallback_policy {
            FallbackPolicy::OnAnyError => true,
            FallbackPolicy::OnAbsence => {
                matches!(
                    kind,
                    ProviderErrorKind::Unsupported | ProviderErrorKind::Unavailable
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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProviderResolver")
            .field("registry", &self.registry)
            .field("fallback_policy", &self.fallback_policy)
            .finish()
    }
}
