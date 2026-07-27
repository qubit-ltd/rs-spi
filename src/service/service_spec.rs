// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Base service specification binding provider configuration.

use std::error::Error;

/// Type-level description of one pluggable service family.
///
/// A service specification gives registries one type parameter carrying the
/// configuration accepted by providers. Creation capabilities select their
/// output independently through [`crate::SyncServiceSpec`] or
/// [`crate::AsyncServiceSpec`]. Define one marker type implementing this trait
/// for each service family that needs independently typed registration.
pub trait ServiceSpec: 'static {
    /// Configuration type passed to provider factories.
    ///
    /// This may be unsized when providers operate on a dynamically sized
    /// configuration view.
    type Config: ?Sized;
    /// Domain error returned when a provider cannot create this service.
    type Error: Error + Send + Sync + 'static;
}
