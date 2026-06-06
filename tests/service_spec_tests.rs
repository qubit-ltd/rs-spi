use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;

use qubit_spi::{
    ProviderCreateError,
    ProviderDescriptor,
    ProviderRegistry,
    ProviderRegistryError,
    ServiceProvider,
    ServiceSpec,
};

/// Service contract used to verify multiple service handle outputs.
trait HandleService: Debug {
    /// Gets the provider-created value.
    fn value(&self) -> usize;
}

/// Concrete handle service used by provider tests.
#[derive(Debug)]
struct HandleServiceImpl {
    /// Value returned through the service trait.
    value: usize,
}

impl HandleService for HandleServiceImpl {
    fn value(&self) -> usize {
        self.value
    }
}

/// Service specification whose service type is the trait object itself.
#[derive(Debug)]
struct HandleSpec;

impl ServiceSpec for HandleSpec {
    type Config = usize;
    type Service = dyn HandleService;
}

/// Provider that can create the same service through different handle types.
#[derive(Debug)]
struct HandleProvider;

impl ServiceProvider<HandleSpec> for HandleProvider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        ProviderDescriptor::new("handle")
    }

    fn create_box(
        &self,
        config: &usize,
    ) -> Result<Box<dyn HandleService>, ProviderCreateError> {
        Ok(Box::new(HandleServiceImpl { value: *config }))
    }
}

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
    type Service = usize;
}

/// Provider that reads values from an unsized configuration reference.
#[derive(Debug)]
struct UnsizedConfigProvider;

impl ServiceProvider<UnsizedConfigSpec> for UnsizedConfigProvider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        ProviderDescriptor::new("unsized")
    }

    fn create_box(
        &self,
        config: &dyn RuntimeConfig,
    ) -> Result<Box<usize>, ProviderCreateError> {
        Ok(Box::new(config.value()))
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
        .create_box("unsized", &config)
        .expect("provider should accept unsized config references");

    assert_eq!(42, *value);
}

/// Test one service specification can create Box, Arc, and Rc service handles.
#[test]
fn test_service_spec_supports_multiple_service_handles() {
    let mut registry = ProviderRegistry::<HandleSpec>::new();
    registry
        .register(HandleProvider)
        .expect("provider descriptor should be valid");

    let boxed: Box<dyn HandleService> = registry
        .create_box("handle", &11)
        .expect("provider should create a boxed service");
    let shared: Arc<dyn HandleService> = registry
        .create_arc("handle", &12)
        .expect("provider should create an atomically shared service");
    let local: Rc<dyn HandleService> = registry
        .create_rc("handle", &13)
        .expect("provider should create a locally shared service");

    assert_eq!(11, boxed.value());
    assert_eq!(12, shared.value());
    assert_eq!(13, local.value());
}
