// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::{CreatedService, ProviderId};

#[test]
fn created_service_contains_the_winning_provider() {
    let created = CreatedService::new(ProviderId::new("memory").unwrap(), 42_u8);

    assert_eq!("memory", created.provider_id().as_str());
    assert_eq!(42, created.into_service());
}

#[test]
fn created_service_decomposes_into_owned_parts() {
    let created = CreatedService::new(ProviderId::new("memory").expect("valid ID"), 42_u8);
    let (provider_id, service) = created.into_parts();

    assert_eq!("memory", provider_id.as_str());
    assert_eq!(42, service);
}
