/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Service specification binding configuration and service contracts.

/// Type-level description of one pluggable service family.
///
/// A service specification gives a registry one type parameter that carries the
/// configuration type accepted by providers and the service contract implemented
/// by produced services.
pub trait ServiceSpec {
    /// Configuration type passed to provider checks and factories.
    type Config: ?Sized;

    /// Service contract implemented by provider-created services.
    type Service: ?Sized;
}
