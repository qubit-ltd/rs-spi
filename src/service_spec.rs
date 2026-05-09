/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Service specification binding configuration and service types.

/// Type-level description of one pluggable service family.
///
/// A service specification gives a registry one type parameter that carries the
/// configuration type accepted by providers and the service type produced by
/// providers.
pub trait ServiceSpec {
    /// Configuration type passed to provider checks and factories.
    type Config: ?Sized;

    /// Service value produced by providers.
    type Service;
}
