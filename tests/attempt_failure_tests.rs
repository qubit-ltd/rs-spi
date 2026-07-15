// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{
    AttemptFailureKind,
    FallbackPolicy,
    ProviderDescriptor,
    ProviderError,
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
    let [attempt] = error.attempts() else {
        panic!("one named provider must produce exactly one attempt");
    };

    assert_eq!(AttemptFailureKind::ProviderError, attempt.kind());
    assert!(attempt.source().is_some());
    assert!(std::error::Error::source(attempt).is_some());
    assert!(attempt.to_string().contains("file-command"));
    assert!(attempt.to_string().contains("file executable is absent"));
}

/// Verifies that an unresolved chain selector has an explicit attempt kind.
#[test]
fn test_unknown_attempt_has_an_explicit_kind() {
    let resolver = ProviderResolver::<FailureSpec>::new(
        ProviderRegistry::default(),
        FallbackPolicy::OnAbsence,
    );
    let error = resolver
        .create_chain(["missing"], &())
        .expect_err("the empty registry cannot resolve the selector");
    let [attempt] = error.attempts() else {
        panic!("one unknown selector must produce exactly one attempt");
    };

    assert_eq!(AttemptFailureKind::UnknownProvider, attempt.kind());
    assert_eq!(
        Some("missing"),
        attempt.requested_selector().map(ProviderSelector::as_str)
    );
    assert!(attempt.provider_id().is_none());
    assert!(attempt.provider_error_kind().is_none());
    assert!(attempt.source().is_none());
}
