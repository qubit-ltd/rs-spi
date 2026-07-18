// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policies controlling fallback after provider creation failures.

/// Controls which provider creation failures permit another candidate.
///
/// This policy applies to automatic and chained selection after a provider
/// factory returns an error. Named selection always uses exactly one provider
/// and never falls back. It does not handle errors produced by a service after
/// that service has been created successfully.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum FallbackPolicy {
    /// Stops after the first provider creation failure.
    Never,
    /// Continues only after an unsupported or unavailable provider.
    #[default]
    OnAbsence,
    /// Continues after every leaf provider creation failure.
    OnAnyError,
}
