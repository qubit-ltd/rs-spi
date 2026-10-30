// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_spi::ServiceSpec;
use qubit_spi::SyncServiceSpec;

/// Service family proving that configuration may be dynamically sized.
struct UnsizedConfigSpec;

impl ServiceSpec for UnsizedConfigSpec {
    type Config = str;
    type Error = std::io::Error;
}

/// Service family proving that synchronous output is a separate capability.
struct StringOutputSpec;

impl ServiceSpec for StringOutputSpec {
    type Config = str;
    type Error = std::io::Error;
}

impl SyncServiceSpec for StringOutputSpec {
    type Output = String;
}

/// Accepts only a base service specification with unsized configuration.
fn assert_unsized_config_spec<S: ServiceSpec<Config = str>>() {}

/// Accepts only a synchronous service specification with string output.
fn assert_string_output_spec<S: SyncServiceSpec<Output = String>>() {}

/// Verifies unsized configuration and sized output associated types.
#[test]
fn test_service_spec_accepts_unsized_configuration_and_sized_output() {
    assert_unsized_config_spec::<UnsizedConfigSpec>();
    assert_unsized_config_spec::<StringOutputSpec>();
    assert_string_output_spec::<StringOutputSpec>();
}

mod capability_boundary_tests {
    use std::io;
    use std::rc::Rc;

    use qubit_spi::AsyncProviderRegistry;
    use qubit_spi::AsyncServiceProvider;
    use qubit_spi::AsyncServiceSpec;
    use qubit_spi::ProviderDescriptor;
    use qubit_spi::ProviderFuture;
    use qubit_spi::ProviderMetadata;
    use qubit_spi::ProviderRegistry;
    use qubit_spi::ServiceProvider;
    use qubit_spi::ServiceSpec;
    use qubit_spi::SyncServiceSpec;
    use qubit_spi::error::ProviderFailure;

    struct TextSpec;

    impl ServiceSpec for TextSpec {
        type Config = str;
        type Error = io::Error;
    }

    impl SyncServiceSpec for TextSpec {
        type Output = Rc<str>;
    }

    impl AsyncServiceSpec for TextSpec {
        type Output = usize;
    }

    struct TextProvider;

    impl ProviderMetadata for TextProvider {
        fn descriptor(&self) -> ProviderDescriptor {
            qubit_spi::provider_descriptor!("text")
        }
    }

    impl ServiceProvider<TextSpec> for TextProvider {
        fn create_configured(&self, config: &str) -> Result<Rc<str>, ProviderFailure<io::Error>> {
            Ok(Rc::from(config))
        }
    }

    impl AsyncServiceProvider<TextSpec> for TextProvider {
        fn create_configured<'a>(
            &'a self,
            config: &'a str,
        ) -> ProviderFuture<'a, Result<usize, ProviderFailure<io::Error>>> {
            Box::pin(async move { Ok(config.len()) })
        }
    }

    fn require_send<T: Send>(value: T) -> T {
        value
    }

    #[test]
    fn test_sync_registry_accepts_unsized_config_and_non_send_output() {
        let registry = ProviderRegistry::<TextSpec>::default();
        registry.register(TextProvider).expect("provider registers");
        let resolver = registry.resolve().expect("provider resolves");
        let input = String::from("borrowed text");
        let output: Rc<str> = resolver
            .create_configured(input.as_str())
            .expect("service creation succeeds");
        assert_eq!(input.as_str(), output.as_ref());
    }

    #[test]
    fn test_async_registry_accepts_unsized_config_and_distinct_output() {
        let registry = AsyncProviderRegistry::<TextSpec>::default();
        registry.register(TextProvider).expect("provider registers");
        let resolver = registry.resolve().expect("provider resolves");
        let input = String::from("borrowed text");
        let future = require_send(resolver.create_configured(input.as_str()));
        let output: usize = futures::executor::block_on(future).expect("service creation succeeds");
        assert_eq!(input.len(), output);
    }
}
