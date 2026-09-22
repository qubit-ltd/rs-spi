use provider_async as _;
use spi::ProviderSelection;

/// Proves that a linked asynchronous provider constructs its service.
fn main() {
    let registry = service_contract::async_providers::build_registry()
        .expect("async provider should register");
    let resolver = registry
        .resolve_selected(
            &ProviderSelection::named("async").expect("async selector should be valid"),
        )
        .expect("async provider should resolve");
    let output =
        futures::executor::block_on(resolver.create()).expect("async provider should create");
    println!("{output}");
}
