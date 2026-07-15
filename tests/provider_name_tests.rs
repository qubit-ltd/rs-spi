use qubit_spi::{ProviderName, ProviderRegistryError};

/// Test provider names implement standard string conversion traits.
#[test]
fn test_provider_name_string_traits() {
    let name: ProviderName = " Native-Provider_1 "
        .parse()
        .expect("provider name should parse");

    assert_eq!("native-provider_1", name.as_ref());
    assert_eq!("native-provider_1", format!("{name}"));
}

/// Test provider names are trimmed and normalized to lowercase ASCII.
#[test]
fn test_new_trims_and_normalizes_provider_names() {
    let name =
        ProviderName::new(" Native-Provider_1 ").expect("provider name should be normalized");

    assert_eq!("native-provider_1", name.as_str());
    assert_eq!("native-provider_1", name.to_string());
}

/// Test empty provider names are rejected.
#[test]
fn test_new_rejects_empty_provider_names() {
    let error = ProviderName::new(" ").expect_err("empty names should fail");

    assert!(matches!(error, ProviderRegistryError::EmptyProviderName));
}

/// Test provider names reject unsupported characters.
#[test]
fn test_new_rejects_invalid_provider_name_characters() {
    let error = ProviderName::new("native provider").expect_err("spaces should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::InvalidProviderName { ref name, .. } if name == "native provider"
    ));
}

/// Test provider names reject dots because config paths may treat them as
/// nesting.
#[test]
fn test_new_rejects_dotted_provider_names() {
    let error = ProviderName::new("native.provider").expect_err("dots should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::InvalidProviderName { ref name, .. } if name == "native.provider"
    ));
}

/// Test provider names must start with an alphanumeric character.
#[test]
fn test_new_rejects_provider_names_starting_with_separator() {
    let error = ProviderName::new("_native").expect_err("leading separators should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::InvalidProviderName { ref name, .. } if name == "_native"
    ));
}

/// Test provider names must end with an alphanumeric character.
#[test]
fn test_new_rejects_provider_names_ending_with_separator() {
    let error = ProviderName::new("native-").expect_err("trailing separators should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::InvalidProviderName { ref name, .. } if name == "native-"
    ));
}

/// Test provider names reject adjacent separators.
#[test]
fn test_new_rejects_consecutive_provider_name_separators() {
    let error =
        ProviderName::new("native-_provider").expect_err("consecutive separators should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::InvalidProviderName { ref name, .. } if name == "native-_provider"
    ));
}

/// Test provider names reject non-ASCII input.
#[test]
fn test_new_rejects_non_ascii_provider_names() {
    let error = ProviderName::new("原生").expect_err("non-ASCII names should fail");

    assert!(matches!(
        error,
        ProviderRegistryError::InvalidProviderName { ref name, .. } if name == "原生"
    ));
}
