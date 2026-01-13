use std::collections::HashMap;

use serde_vars::{source::Source, MapSource};

#[test]
fn test_multiple_sources() {
    #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
    struct Test {
        f1: String,
        f2: String,
        f3: String,
    }

    let s1 = MapSource::new(HashMap::from([("FOO".to_owned(), "foo_s1".to_owned())]))
        .with_variable_prefix("${s1:");
    let s2 = MapSource::new(HashMap::from([("FOO".to_owned(), "foo_s2".to_owned())]))
        .with_variable_prefix("${s2:");
    let mut source = (s1, s2);

    let x = r#"{
        "f1": "${s1:FOO}",
        "f2": "${s2:FOO}",
        "f3": "just some string"
    }"#;
    let mut de = serde_json::Deserializer::from_str(x);

    let s: Test = serde_vars::deserialize(&mut de, &mut source).unwrap();
    assert_eq!(
        s,
        Test {
            f1: "foo_s1".to_owned(),
            f2: "foo_s2".to_owned(),
            f3: "just some string".to_owned(),
        }
    );
}

#[test]
fn test_multiple_sources_do_not_resolve_multiple_times() {
    let s1 = MapSource::new(HashMap::from([("FOO".to_owned(), "${s2:FOO}".to_owned())]))
        .with_variable_prefix("${s1:");
    let s2 = MapSource::new(HashMap::from([("FOO".to_owned(), "foo_s2".to_owned())]))
        .with_variable_prefix("${s2:");
    let mut source = (s1, s2);

    let mut de = serde_json::Deserializer::from_str("\"${s1:FOO}\"");

    let s: String = serde_vars::deserialize(&mut de, &mut source).unwrap();
    assert_eq!(s, "${s2:FOO}");
}

#[test]
fn test_multiple_sources_first_one_wins() {
    let s1 = MapSource::new(HashMap::from([("FOO".to_owned(), "foo_s1".to_owned())]));
    let s2 = MapSource::new(HashMap::from([
        ("FOO".to_owned(), "foo_s2".to_owned()),
        ("BAR".to_owned(), "bar_s2".to_owned()),
    ]));
    let mut source = (s1, s2);

    let mut de = serde_json::Deserializer::from_str("\"${FOO}\"");
    let s: String = serde_vars::deserialize(&mut de, &mut source).unwrap();
    assert_eq!(s, "foo_s1");

    let mut de = serde_json::Deserializer::from_str("\"${BAR}\"");
    let err: Result<String, _> = serde_vars::deserialize(&mut de, &mut source);
    assert_eq!(
        format!("{:?}", err.unwrap_err()),
        "Error(\"got variable `${BAR}`, but it does not exist\", line: 0, column: 0)"
    );
}

#[test]
fn test_tuple_is_source() {
    fn is_source<T: Source>() {}

    fn some_source<T: Source>() {
        is_source::<(T,)>();
        is_source::<(T, T)>();
        is_source::<(T, T, T)>();
        is_source::<(T, T, T, T)>();
        is_source::<(T, T, T, T, T)>();
        is_source::<(T, T, T, T, T, T)>();
        is_source::<(T, T, T, T, T, T, T)>();
        is_source::<(T, T, T, T, T, T, T, T)>();
        is_source::<(T, T, T, T, T, T, T, T, T)>();
        is_source::<(T, T, T, T, T, T, T, T, T, T)>();
        is_source::<(T, T, T, T, T, T, T, T, T, T, T)>();
        is_source::<(T, T, T, T, T, T, T, T, T, T, T, T)>();
        is_source::<(T, T, T, T, T, T, T, T, T, T, T, T, T)>();
        is_source::<(T, T, T, T, T, T, T, T, T, T, T, T, T, T)>();
        is_source::<(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T)>();
    }

    some_source::<MapSource>();
}
