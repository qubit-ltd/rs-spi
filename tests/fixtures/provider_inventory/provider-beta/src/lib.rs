//! Synchronous beta provider that consumers deliberately leave unlinked.

use std::convert::Infallible;

use spi::ProviderDescriptor;
use spi::ProviderMetadata;
use spi::ServiceProvider;
use spi::error::ProviderFailure;

/// Provider producing the beta fixture output.
struct BetaProvider;

impl ProviderMetadata for BetaProvider {
    /// Returns beta's stable registration descriptor.
    fn descriptor(&self) -> ProviderDescriptor {
        spi::provider_descriptor!("beta")
    }
}

impl ServiceProvider<service_contract::FixtureSpec> for BetaProvider {
    /// Creates the beta fixture service output.
    fn create_configured(&self, _config: &String) -> Result<String, ProviderFailure<Infallible>> {
        Ok("beta".to_owned())
    }
}

spi::submit_sync_provider! {
    inventory_entry = service_contract::sync_providers::Entry;
    spec = service_contract::FixtureSpec;
    provider = BetaProvider;
}
