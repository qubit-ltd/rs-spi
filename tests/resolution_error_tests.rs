// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{
    AttemptFailure, AttemptFailureKind, ProviderError, ProviderId, ProviderSelector,
    ResolutionError, ResolutionErrorKind,
};

#[test]
fn resolution_error_exposes_its_kind_and_attempts() {
    let error = ResolutionError::unknown_provider("missing");

    assert_eq!(ResolutionErrorKind::UnknownProvider, error.kind());
    assert!(error.attempts().is_empty());
}

#[test]
fn attempt_failure_preserves_provider_error_source() {
    let error = ProviderError::unavailable_with_source(
        "file executable is absent",
        std::io::Error::other("ENOENT"),
    );
    let attempt =
        AttemptFailure::provider_error(None, ProviderId::new("file-command").unwrap(), &error);

    assert_eq!(AttemptFailureKind::ProviderError, attempt.kind());
    assert!(attempt.source().is_some());
}

#[test]
fn unknown_attempt_has_an_explicit_kind() {
    let attempt = AttemptFailure::unknown_provider(ProviderSelector::parse("missing").unwrap());

    assert_eq!(AttemptFailureKind::UnknownProvider, attempt.kind());
    assert_eq!(
        Some("missing"),
        attempt.requested_selector().map(ProviderSelector::as_str)
    );
    assert!(attempt.provider_id().is_none());
    assert!(attempt.provider_error_kind().is_none());
    assert!(attempt.source().is_none());
}
