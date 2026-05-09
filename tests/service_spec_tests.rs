use std::fmt::Debug;

use qubit_spi::{
    ProviderCreateError,
    ProviderDescriptor,
    ProviderRegistry,
    ProviderRegistryError,
    ServiceProvider,
    ServiceSpec,
};

/// Runtime configuration contract used to verify unsized config support.
trait RuntimeConfig {
    /// Gets the configured value.
    fn value(&self) -> usize;
}

/// Concrete runtime configuration for tests.
struct TestRuntimeConfig {
    /// Value exposed through the runtime config trait.
    value: usize,
}

impl RuntimeConfig for TestRuntimeConfig {
    fn value(&self) -> usize {
        self.value
    }
}

/// Service specification whose configuration type is a trait object.
#[derive(Debug)]
struct UnsizedConfigSpec;

impl ServiceSpec for UnsizedConfigSpec {
    type Config = dyn RuntimeConfig;
    type Output = usize;
}

/// Provider that reads values from an unsized configuration reference.
#[derive(Debug)]
struct UnsizedConfigProvider;

impl ServiceProvider<UnsizedConfigSpec> for UnsizedConfigProvider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        ProviderDescriptor::new("unsized")
    }

    fn create(&self, config: &dyn RuntimeConfig) -> Result<usize, ProviderCreateError> {
        Ok(config.value())
    }
}

/// Test service specs can use an unsized configuration type.
#[test]
fn test_service_spec_accepts_unsized_config() {
    let mut registry = ProviderRegistry::<UnsizedConfigSpec>::new();
    registry
        .register(UnsizedConfigProvider)
        .expect("provider descriptor should be valid");
    let config = TestRuntimeConfig { value: 42 };

    let value = registry
        .create("unsized", &config)
        .expect("provider should accept unsized config references");

    assert_eq!(42, value);
}
