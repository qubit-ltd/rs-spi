// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Regression tests for typed-failure documentation examples.

/// Verifies every shipped guide uses the typed provider failure API.
#[test]
fn test_shipped_guides_use_typed_provider_failure_api() {
    for (name, guide) in [
        ("README.md", include_str!("../README.md")),
        ("README.zh_CN.md", include_str!("../README.zh_CN.md")),
        ("doc/user_guide.md", include_str!("../doc/user_guide.md")),
        ("doc/user_guide.zh_CN.md", include_str!("../doc/user_guide.zh_CN.md")),
    ] {
        assert!(
            !guide.contains("ProviderError"),
            "{name} must not mention the removed ProviderError API"
        );
        assert!(
            guide.contains("ProviderFailure<GreeterError>"),
            "{name} must demonstrate a typed provider failure"
        );
        assert!(
            guide.contains("type Error = GreeterError;"),
            "{name} must declare ServiceSpec::Error"
        );
    }
}

/// Verifies the provider-input fuzz target bounds bytes before UTF-8 parsing.
#[test]
fn test_provider_input_fuzz_target_bounds_input_before_parsing() {
    let target = include_str!("../fuzz/fuzz_targets/provider_input.rs");
    let bound = target
        .find("const MAX_INPUT_BYTES: usize = 4096;")
        .expect("provider-input fuzz target must declare the CI-aligned byte bound");
    let guard = target
        .find("if data.len() > MAX_INPUT_BYTES {")
        .expect("provider-input fuzz target must reject oversized inputs");
    let parse = target
        .find("str::from_utf8(data)")
        .expect("provider-input fuzz target must parse its byte input as UTF-8");

    assert!(bound < guard, "the byte bound must be declared before its guard");
    assert!(guard < parse, "the byte guard must run before UTF-8 parsing");
}
