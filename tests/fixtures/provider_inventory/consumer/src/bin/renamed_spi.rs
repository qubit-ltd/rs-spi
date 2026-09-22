use provider_alpha as _;
use provider_beta as _;
use spi::ProviderSelection;

/// Proves macro-generated code remains valid when qubit-spi is named `spi`.
fn main() {
    let registry =
        service_contract::sync_providers::build_registry().expect("alpha should register");
    assert_eq!(
        vec!["alpha", "beta"],
        registry
            .provider_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        "two independently linked provider crates should share the collection"
    );
    let resolver = registry
        .resolve_selected(
            &ProviderSelection::named("alpha").expect("alpha selector should be valid"),
        )
        .expect("alpha should resolve through renamed dependency");
    assert_eq!("alpha", resolver.create().expect("alpha should create"));
    println!("alpha,beta");
}
