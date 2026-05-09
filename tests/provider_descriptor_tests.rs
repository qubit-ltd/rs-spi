use qubit_spi::{
    ProviderDescriptor,
    ProviderRegistryError,
};

/// Test provider descriptors normalize ids, aliases, and priority.
#[test]
fn test_new_with_aliases_and_priority_normalizes_metadata() {
    let descriptor = ProviderDescriptor::new(" Native ")
        .expect("provider id should be valid")
        .with_aliases(&[" FAST ", "default"])
        .expect("aliases should be valid")
        .with_priority(10);

    assert_eq!("native", descriptor.id().as_str());
    assert_eq!(vec!["fast", "default"], descriptor.aliases_as_str());
    assert_eq!(10, descriptor.priority());
}

/// Test provider descriptors reject invalid aliases.
#[test]
fn test_with_aliases_rejects_invalid_aliases() {
    let error = ProviderDescriptor::new("native")
        .expect("provider id should be valid")
        .with_aliases(&["bad alias"])
        .expect_err("invalid aliases should be rejected");

    assert!(matches!(
        error,
        ProviderRegistryError::InvalidProviderName { ref name, .. } if name == "bad alias"
    ));
}

/// Test descriptors reject duplicate names before registration.
#[test]
fn test_with_aliases_rejects_duplicate_descriptor_names() {
    let duplicate_id_error = ProviderDescriptor::new("native")
        .expect("provider id should be valid")
        .with_aliases(&["native"])
        .expect_err("alias matching id should fail");
    let duplicate_alias_error = ProviderDescriptor::new("native")
        .expect("provider id should be valid")
        .with_aliases(&["fast", "FAST"])
        .expect_err("duplicate aliases should fail");

    assert!(matches!(
        duplicate_id_error,
        ProviderRegistryError::DuplicateProviderName { ref name }
            if name.as_str() == "native"
    ));
    assert!(matches!(
        duplicate_alias_error,
        ProviderRegistryError::DuplicateProviderName { ref name }
            if name.as_str() == "fast"
    ));
}
