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
/// and never falls back.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FallbackPolicy {
    /// Continues only after an unsupported or unavailable provider.
    #[default]
    OnAbsence,
    /// Continues after every provider creation failure.
    OnAnyError,
}
