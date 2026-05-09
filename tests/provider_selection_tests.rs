use qubit_spi::ProviderSelection;

/// Test the default selection uses automatic provider selection.
#[test]
fn test_default_selection_uses_auto_without_fallbacks() {
    let selection = ProviderSelection::default();

    assert_eq!("auto", selection.default_name());
    assert_eq!("auto", selection.auto_name());
    assert!(selection.fallbacks().is_empty());
    assert!(selection.is_auto_default());
}

/// Test name-based selection trims default and fallback names.
#[test]
fn test_from_names_trims_default_and_fallback_names() {
    let selection = ProviderSelection::from_names(" native ", &[" fallback ", "", " backup "]);

    assert_eq!("native", selection.default_name());
    assert_eq!(
        &["fallback".to_owned(), "backup".to_owned()],
        selection.fallbacks(),
    );
    assert!(!selection.is_auto_default());
}

/// Test a custom automatic selector can be configured.
#[test]
fn test_with_auto_name_changes_auto_keyword() {
    let selection = ProviderSelection::from_names("best", &[]).with_auto_name("best");

    assert_eq!("best", selection.auto_name());
    assert!(selection.is_auto_default());
}

/// Test owned fallback names are normalized.
#[test]
fn test_new_normalizes_owned_fallback_names() {
    let fallbacks = vec![
        " fallback ".to_owned(),
        "".to_owned(),
        " backup ".to_owned(),
    ];
    let selection = ProviderSelection::new("native", &fallbacks);

    assert_eq!("native", selection.default_name());
    assert_eq!(
        &["fallback".to_owned(), "backup".to_owned()],
        selection.fallbacks(),
    );
}

/// Test empty auto keywords fall back to the standard keyword.
#[test]
fn test_with_auto_name_normalizes_empty_keyword_to_auto() {
    let selection = ProviderSelection::from_names("auto", &[]).with_auto_name(" ");

    assert_eq!("auto", selection.auto_name());
    assert!(selection.is_auto_default());
}
