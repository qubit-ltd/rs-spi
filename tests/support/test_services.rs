#![allow(dead_code)]

use std::fmt::Debug;
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use qubit_spi::{
    ProviderAvailability,
    ProviderRegistry,
    ProviderRegistryError,
    ServiceProvider,
};

/// Configuration used by test providers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestConfig {
    prefix: String,
}

impl TestConfig {
    /// Creates a test configuration.
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_owned(),
        }
    }

    /// Gets the message prefix.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

/// Service trait created by test providers.
pub trait GreetingService: Debug + Send + Sync {
    /// Builds the greeting text.
    fn greet(&self) -> String;
}

/// Concrete greeting service used by tests.
#[derive(Debug)]
struct StaticGreetingService {
    message: String,
}

impl GreetingService for StaticGreetingService {
    fn greet(&self) -> String {
        self.message.clone()
    }
}

/// Provider registry type used by greeting tests.
pub type GreetingRegistry = ProviderRegistry<dyn GreetingService, TestConfig>;

/// Provider implementation used by registry tests.
#[derive(Debug)]
pub struct GreetingProvider {
    id: &'static str,
    aliases: &'static [&'static str],
    message: &'static str,
    priority: i32,
    availability: ProviderAvailability,
    failure: Option<ProviderRegistryError>,
    created: AtomicUsize,
}

impl GreetingProvider {
    /// Creates an available greeting provider.
    pub fn new(id: &'static str, message: &'static str) -> Self {
        Self {
            id,
            aliases: &[],
            message,
            priority: 0,
            availability: ProviderAvailability::Available,
            failure: None,
            created: AtomicUsize::new(0),
        }
    }

    /// Sets aliases for this provider.
    pub fn with_aliases(mut self, aliases: &'static [&'static str]) -> Self {
        self.aliases = aliases;
        self
    }

    /// Sets priority for automatic provider selection.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Makes this provider unavailable.
    pub fn unavailable(mut self, reason: &str) -> Self {
        self.availability = ProviderAvailability::unavailable(reason);
        self
    }

    /// Makes this provider fail during creation.
    pub fn failing(mut self, message: &'static str) -> Self {
        self.failure = Some(ProviderRegistryError::create_failed(message));
        self
    }

    /// Makes this provider fail with a specific registry error during creation.
    pub fn failing_with(mut self, error: ProviderRegistryError) -> Self {
        self.failure = Some(error);
        self
    }

    /// Gets how many services this provider created.
    pub fn created_count(&self) -> usize {
        self.created.load(Ordering::SeqCst)
    }
}

impl ServiceProvider for GreetingProvider {
    type Config = TestConfig;
    type Service = dyn GreetingService;

    fn id(&self) -> &'static str {
        self.id
    }

    fn aliases(&self) -> &'static [&'static str] {
        self.aliases
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn availability(&self, _config: &Self::Config) -> ProviderAvailability {
        self.availability.clone()
    }

    fn create(&self, config: &Self::Config) -> Result<Box<Self::Service>, ProviderRegistryError> {
        self.created.fetch_add(1, Ordering::SeqCst);
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        Ok(Box::new(StaticGreetingService {
            message: format!("{}{}", config.prefix(), self.message),
        }))
    }
}
