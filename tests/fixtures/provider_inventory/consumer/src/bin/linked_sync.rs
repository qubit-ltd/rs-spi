use provider_alpha as _;
use spi::ProviderSelection;

/// Builds the synchronous registry after explicitly anchoring alpha.
fn main() {
    let registry =
        service_contract::sync_providers::build_registry().expect("alpha should register");
    let resolver = registry
        .resolve_selected(
            &ProviderSelection::named("alpha").expect("alpha selector should be valid"),
        )
        .expect("alpha should resolve");
    println!("{}", resolver.create().expect("alpha should create"));
}
