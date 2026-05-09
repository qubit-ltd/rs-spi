mod support;

use qubit_spi::{
    ProviderAvailability,
    ProviderCreateError,
    ProviderDescriptor,
    ProviderRegistryError,
    ServiceProvider,
    ServiceSpec,
};

use crate::support::test_services::{
    GreetingProvider,
    GreetingSpec,
    TestConfig,
};

/// Minimal provider used to exercise default trait methods.
#[derive(Debug)]
struct MinimalProvider;

impl ServiceProvider<GreetingSpec> for MinimalProvider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        ProviderDescriptor::new("minimal")
    }

    fn create_box(
        &self,
        config: &TestConfig,
    ) -> Result<Box<<GreetingSpec as ServiceSpec>::Service>, ProviderCreateError> {
        GreetingProvider::new("delegate", "hello").create_box(config)
    }
}

/// Test default provider methods supply common availability defaults.
#[test]
fn test_provider_default_methods_return_available() {
    let provider = MinimalProvider;
    let availability = provider.availability(&TestConfig::new(""));
    let descriptor = provider
        .descriptor()
        .expect("minimal provider descriptor should be valid");

    assert_eq!("minimal", descriptor.id().as_str());
    assert!(descriptor.aliases().is_empty());
    assert_eq!(0, descriptor.priority());
    assert_eq!(ProviderAvailability::Available, availability);
}

/// Test provider creation receives the caller configuration.
#[test]
fn test_provider_create_uses_config() {
    let provider = GreetingProvider::new("static", "hello");
    let service = provider
        .create_box(&TestConfig::new("say "))
        .expect("provider should create a greeting service");

    assert_eq!("say hello", service.greet());
    assert_eq!(1, provider.created_count());
}
