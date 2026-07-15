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
pub struct ProviderResolver<S>
where
    S: ServiceSpec,
{
    registry: ProviderRegistry<S>,
    fallback_policy: FallbackPolicy,
}

impl<S> ProviderResolver<S>
where
    S: ServiceSpec,
{
    /// Creates a resolver over an immutable provider registry.
    #[must_use]
    pub fn new(registry: ProviderRegistry<S>, fallback_policy: FallbackPolicy) -> Self {
        Self {
            registry,
            fallback_policy,
        }
    }

    /// Creates a service using the requested selection.
    ///
    /// # Errors
    ///
    /// Returns ResolutionError when no candidate succeeds or a non-fallback
    /// provider failure occurs.
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
