use qubit_spi::{
    ProviderName,
    ProviderRegistryError,
    ProviderSelection,
};

/// Creates a provider name used by selection assertions.
fn name(value: &str) -> ProviderName {
    ProviderName::new(value).expect("test provider name should be valid")
}

/// Test the default selection uses automatic provider selection.
#[test]
fn test_default_selection_uses_auto_without_fallbacks() {
    let selection = ProviderSelection::auto();

    assert!(selection.is_auto());
    assert!(selection.primary().is_none());
    assert!(selection.fallbacks().is_empty());
    assert_eq!(ProviderSelection::default(), selection);
}

/// Test name-based selection trims and normalizes provider names.
#[test]
fn test_from_names_trims_and_normalizes_names() {
    let selection = ProviderSelection::from_names(" Native ", &[" Fallback ", " backup "])
        .expect("selection names should be valid");

    assert!(!selection.is_auto());
    assert_eq!(Some(&name("native")), selection.primary());
    assert_eq!(&[name("fallback"), name("backup")], selection.fallbacks());
}

/// Test empty fallback names are rejected instead of being silently ignored.
#[test]
fn test_from_names_rejects_empty_fallback_names() {
    let error = ProviderSelection::from_names("native", &[" "])
        .expect_err("empty fallback names should be rejected");

    assert!(matches!(error, ProviderRegistryError::EmptyProviderName));
}

/// Test invalid provider names are rejected at selection construction.
#[test]
fn test_from_names_rejects_invalid_provider_names() {
    let error = ProviderSelection::from_names("native provider", &[])
        .expect_err("names with spaces should be rejected");

    assert!(matches!(
        error,
        ProviderRegistryError::InvalidProviderName { ref name, .. } if name == "native provider"
    ));
}

/// Test duplicated candidate names are rejected at selection construction.
#[test]
fn test_from_names_rejects_duplicate_candidates() {
    let primary_error = ProviderSelection::from_names("native", &["NATIVE"])
        .expect_err("fallbacks must not repeat the primary provider");
    let fallback_error = ProviderSelection::from_names("native", &["fallback", "FALLBACK"])
        .expect_err("fallbacks must not repeat earlier fallback providers");

    assert!(matches!(
        primary_error,
        ProviderRegistryError::DuplicateProviderName { ref name } if name.as_str() == "native"
    ));
    assert!(matches!(
        fallback_error,
        ProviderRegistryError::DuplicateProviderName { ref name } if name.as_str() == "fallback"
    ));
}

/// Test explicit named selections can be built without fallbacks.
#[test]
fn test_named_selection_has_primary_without_fallbacks() {
    let selection = ProviderSelection::named("native").expect("selection name should be valid");

    assert!(!selection.is_auto());
    assert_eq!(Some(&name("native")), selection.primary());
    assert!(selection.fallbacks().is_empty());
}

/// Test owned fallback names use the same normalization rules.
#[test]
fn test_from_owned_names_normalizes_owned_fallback_names() {
    let fallbacks = vec![" Fallback ".to_owned(), " BACKUP ".to_owned()];
    let selection = ProviderSelection::from_owned_names(" Native ", &fallbacks)
        .expect("selection names should be valid");

    assert_eq!(Some(&name("native")), selection.primary());
    assert_eq!(&[name("fallback"), name("backup")], selection.fallbacks());
}

/// Test owned fallback names reject invalid values.
#[test]
fn test_from_owned_names_rejects_invalid_owned_fallback_names() {
    let fallbacks = vec!["fallback provider".to_owned()];
    let error = ProviderSelection::from_owned_names("native", &fallbacks)
        .expect_err("invalid fallback names should be rejected");

    assert!(matches!(
        error,
        ProviderRegistryError::InvalidProviderName { ref name, .. }
            if name == "fallback provider"
    ));
}

/// Test duplicated owned candidate names are rejected.
#[test]
fn test_from_owned_names_rejects_duplicate_owned_candidates() {
    let fallbacks = vec!["fallback".to_owned(), " FALLBACK ".to_owned()];
    let error = ProviderSelection::from_owned_names("native", &fallbacks)
        .expect_err("owned fallback names should be unique");

    assert!(matches!(
        error,
        ProviderRegistryError::DuplicateProviderName { ref name } if name.as_str() == "fallback"
    ));
}
