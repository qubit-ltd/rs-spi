mod support;

use qubit_spi::{
    ProviderAvailability,
    ServiceProvider,
};

use crate::support::test_services::{
    GreetingProvider,
    GreetingService,
    TestConfig,
    TestProviderError,
};

/// Minimal provider used to exercise default trait methods.
#[derive(Debug)]
struct MinimalProvider;

impl ServiceProvider for MinimalProvider {
    type Config = TestConfig;
    type Error = TestProviderError;
    type Service = dyn GreetingService;

    fn id(&self) -> &'static str {
        "minimal"
    }

    fn create(&self, config: &Self::Config) -> Result<Box<Self::Service>, Self::Error> {
        GreetingProvider::new("delegate", "hello").create(config)
    }
}

/// Test default provider methods supply common metadata defaults.
#[test]
fn test_provider_default_methods_return_empty_aliases_zero_priority_and_available() {
    let provider = MinimalProvider;
    let availability = provider.availability(&TestConfig::new(""));

    assert_eq!("minimal", provider.id());
    assert_eq!(&[] as &[&str], provider.aliases());
    assert_eq!(0, provider.priority());
    assert_eq!(ProviderAvailability::Available, availability);
}

/// Test provider creation receives the caller configuration.
#[test]
fn test_provider_create_uses_config() {
    let provider = GreetingProvider::new("static", "hello");
    let service = provider
        .create(&TestConfig::new("say "))
        .expect("provider should create a greeting service");

    assert_eq!("say hello", service.greet());
    assert_eq!(1, provider.created_count());
}
