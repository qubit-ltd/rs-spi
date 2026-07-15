// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy-driven provider selection and service creation.

use std::collections::HashSet;

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
        match selection {
            ProviderSelection::Auto => self.create_automatic(config),
            ProviderSelection::Named(selector) => self.create_named(selector, config),
            ProviderSelection::Chain(selectors) => self.create_chain(selectors, config),
        }
    }

    /// Creates a service by trying all providers in automatic order.
    ///
    /// `config` is forwarded to each candidate. Returns the first successful
    /// service or a [`ResolutionError`] containing every recorded failure.
    fn create_automatic(
        &self,
        config: &S::Config,
    ) -> Result<CreatedService<S::Output>, ResolutionError> {
        let mut failures = Vec::new();
        for &index in self.registry.automatic_indices() {
            match self.registry.resolved_at(index).create(config) {
                Ok(service) => {
                    let provider_id = self.registry.resolved_at(index).descriptor().id().clone();
                    return Ok(CreatedService::new(provider_id, service));
                }
                Err(error) => {
                    let provider_id = self.registry.resolved_at(index).descriptor().id().clone();
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
    fn create_named(
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
    /// only once; `config` is forwarded to each attempted factory. Returns the
    /// first success or a [`ResolutionError`] containing skipped and failed
    /// attempts.
    fn create_chain(
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
    #[inline]
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
