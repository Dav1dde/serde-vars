use serde_vars::MapSource;

/// Yaml allows deserializing numbers (scalars) as strings.
/// This behaviour should not be broken by wrapping the deserializer with `serde-vars`.
#[test]
fn test_yaml_scalar_number_as_string() {
    assert_eq!(serde_yaml::from_str::<String>("300").unwrap(), "300");

    let mut source = MapSource::default();
    let de = serde_yaml::Deserializer::from_str("300");

    let s: String = serde_vars::deserialize(de, &mut source).unwrap();
    assert_eq!(s, "300");
}

/// Same as [`test_yaml_scalar_number_as_string`], just now make sure, number to number works as well.
#[test]
fn test_yaml_scalar_number_as_number() {
    assert_eq!(serde_yaml::from_str::<i32>("300").unwrap(), 300);

    let mut source = MapSource::default();
    let de = serde_yaml::Deserializer::from_str("300");

    let s: i32 = serde_vars::deserialize(de, &mut source).unwrap();
    assert_eq!(s, 300);
}
