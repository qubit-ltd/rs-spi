// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::error::{
    AttemptFailure,
    AttemptFailureKind,
    ProviderError,
    ProviderErrorKind,
    ResolutionError,
};
use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderResolver,
    ProviderSelector,
    ServiceProvider,
    ServiceSpec,
};

/// Service family used to produce provider and lookup attempt failures.
struct FailureSpec;

impl ServiceSpec for FailureSpec {
    type Config = ();
    type Output = ();
}

/// Provider that always returns an unavailable error with a source.
struct FailingProvider;

impl ServiceProvider<FailureSpec> for FailingProvider {
    /// Returns the provider failure retained by the resolver attempt.
    ///
    /// # Arguments
    ///
    /// * `_config` - Unused failure-test configuration.
    ///
    /// # Errors
    ///
    /// Always returns an unavailable [`ProviderError`] with an IO source.
    fn create(&self, _config: &()) -> Result<(), ProviderError> {
        Err(ProviderError::unavailable_with_source(
            "file executable is absent",
            std::io::Error::other("ENOENT"),
        ))
    }
}

/// Creates a resolver containing the deterministic failing provider.
///
/// # Returns
///
/// A resolver configured to stop only after absence-class failures are
/// exhausted.
fn create_failing_resolver() -> ProviderResolver<FailureSpec> {
    let mut builder = ProviderRegistry::<FailureSpec>::builder();
    builder
        .register(
            ProviderDescriptor::new(
                ProviderId::new("file-command")
                    .expect("test provider ID should be valid"),
            ),
            FailingProvider,
        )
        .expect("test provider should register");
    ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence)
}

/// Verifies that aggregate attempts preserve provider error context and source.
#[test]
fn test_attempt_failure_preserves_provider_error_source() {
    let error = create_failing_resolver()
        .create_named("file-command", &())
        .expect_err("the test provider always fails");
    let aggregate_source = std::error::Error::source(&error)
        .and_then(|source| source.downcast_ref::<AttemptFailure>())
        .expect("a single failed attempt should be the aggregate source");
    assert!(std::error::Error::source(aggregate_source).is_some());
    let ResolutionError::NoProviderSucceeded { attempts, .. } = error else {
        panic!("one provider failure should produce an aggregate error");
    };
    let [attempt] = attempts.as_ref() else {
        panic!("one named provider must produce exactly one provider attempt");
    };
    let provider_error = std::error::Error::source(attempt)
        .and_then(|source| source.downcast_ref::<ProviderError>())
        .expect("provider attempts should expose their ProviderError source");
    assert!(std::error::Error::source(provider_error).is_some());
    assert!(attempt.to_string().contains("file-command"));
    assert!(attempt.to_string().contains("file executable is absent"));
    let AttemptFailure::ProviderError {
        requested_selector,
        provider_id,
        error,
    } = attempt
    else {
        panic!("named resolution should retain a provider failure");
    };

    assert_eq!(
        Some("file-command"),
        requested_selector.as_ref().map(ProviderSelector::as_str),
    );
    assert_eq!("file-command", provider_id.as_str());
    assert_eq!(ProviderErrorKind::Unavailable, error.kind());
    assert_eq!("file executable is absent", error.reason());
    assert!(std::error::Error::source(error).is_some());
    assert_eq!(AttemptFailureKind::ProviderError, attempt.kind());
    assert_eq!(
        Some("file-command"),
        attempt.requested_selector().map(ProviderSelector::as_str),
    );
    assert_eq!(
        Some("file-command"),
        attempt.provider_id().map(ProviderId::as_str),
    );
    assert_eq!(
        Some(ProviderErrorKind::Unavailable),
        attempt.provider_error().map(ProviderError::kind),
    );
}

/// Verifies that an unresolved chain selector uses the unknown-provider
/// variant.
#[test]
fn test_unknown_attempt_exposes_requested_selector() {
    let resolver = ProviderResolver::<FailureSpec>::new(
        ProviderRegistry::default(),
        FallbackPolicy::OnAbsence,
    );
    let error = resolver
        .create_chain(["missing"], &())
        .expect_err("the empty registry cannot resolve the selector");
    let ResolutionError::NoProviderSucceeded { attempts, .. } = error else {
        panic!("one unknown selector should produce an aggregate error");
    };
    let [attempt] = attempts.as_ref() else {
        panic!("one unknown selector must produce exactly one attempt");
    };
    assert!(std::error::Error::source(attempt).is_none());
    assert_eq!("unknown provider: missing", attempt.to_string());
    let AttemptFailure::UnknownProvider { requested_selector } = attempt else {
        panic!("unknown selector should retain a lookup failure");
    };

    assert_eq!("missing", requested_selector.as_str());
    assert_eq!(AttemptFailureKind::UnknownProvider, attempt.kind());
    assert_eq!(
        Some("missing"),
        attempt.requested_selector().map(ProviderSelector::as_str),
    );
    assert!(attempt.provider_id().is_none());
    assert!(attempt.provider_error().is_none());
}

/// Verifies automatic provider attempts omit explicit selector context.
#[test]
fn test_automatic_attempt_omits_requested_selector() {
    let error = create_failing_resolver()
        .create_auto(&())
        .expect_err("the test provider always fails");
    let ResolutionError::NoProviderSucceeded { attempts, .. } = error else {
        panic!("automatic provider failure should produce an aggregate error");
    };
    let [attempt] = attempts.as_ref() else {
        panic!("one automatic provider must produce exactly one attempt");
    };

    assert!(!attempt.to_string().contains("requested as"));
    assert!(std::error::Error::source(attempt).is_some());
    let AttemptFailure::ProviderError {
        requested_selector, ..
    } = attempt
    else {
        panic!("automatic resolution should retain a provider failure");
    };
    assert!(requested_selector.is_none());
}
