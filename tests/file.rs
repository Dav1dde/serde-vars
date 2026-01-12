use serde_vars::FileSource;

macro_rules! test_lookup {
    ($name:ident, $value:literal, $ty:ty) => {
        test_lookup!($name, $value, $ty, $value);
    };
    ($name:ident, $value:literal, $ty:ty, $expected:literal) => {
        #[test]
        fn $name() {
            let tempdir = tempfile::tempdir().unwrap();
            std::fs::write(tempdir.path().join("my_test.file"), $value.to_string()).unwrap();

            let mut source = FileSource::new().with_base_path(tempdir.path());
            let mut de = serde_json::Deserializer::from_str("\"${my_test.file}\"");

            let s: $ty = serde_vars::deserialize(&mut de, &mut source).unwrap();
            assert_eq!(s, $expected);
        }
    };
}

test_lookup!(test_lookup_string, "bAr", String);
// This source does not unwrap `"`.
test_lookup!(test_lookup_string_wrapped, "\"bAr\"", String, "\"bAr\"");
test_lookup!(test_lookup_string_int_wrapped, "\"123\"", String, "\"123\"");
test_lookup!(test_lookup_i8, -2, i8);
test_lookup!(test_lookup_i16, -200, i16);
test_lookup!(test_lookup_i32, -2000, i32);
test_lookup!(test_lookup_i64, -20000, i64);
test_lookup!(test_lookup_u8, 20, u8);
test_lookup!(test_lookup_u16, 200, u16);
test_lookup!(test_lookup_u32, 2000, u32);
test_lookup!(test_lookup_u64, 20000, u64);
test_lookup!(test_lookup_f32, 1.0, f32);
test_lookup!(test_lookup_f64, 2.0, f64);

macro_rules! test_missing {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            let mut source = FileSource::new().with_base_path("/does/not/exist/");
            let mut de = serde_json::Deserializer::from_str("\"${does_not.exist}\"");

            let err: Result<$ty, _> = serde_vars::deserialize(&mut de, &mut source);
            assert_eq!(
                &format!("{:?}", err.unwrap_err()),
                r#"Error("failed to read file `/does/not/exist/does_not.exist` from variable `${does_not.exist}`: No such file or directory (os error 2)", line: 0, column: 0)"#
            );
        }
    };
}

test_missing!(test_missing_string, String);
test_missing!(test_missing_i8, i8);
test_missing!(test_missing_i16, i16);
test_missing!(test_missing_i32, i32);
test_missing!(test_missing_i64, i64);
test_missing!(test_missing_u8, u8);
test_missing!(test_missing_u16, u16);
test_missing!(test_missing_u32, u32);
test_missing!(test_missing_u64, u64);
test_missing!(test_missing_f32, f32);
test_missing!(test_missing_f64, f64);

macro_rules! test_invalid {
    ($name:ident, $ty:ty, $value:literal, $err:expr) => {
        #[test]
        fn $name() {
            let tempdir = tempfile::tempdir().unwrap();
            std::fs::write(tempdir.path().join("my_test.file"), $value.to_string()).unwrap();

            let mut source = FileSource::new().with_base_path(tempdir.path());
            let mut de = serde_json::Deserializer::from_str("\"${my_test.file}\"");

            let err: Result<$ty, _> = serde_vars::deserialize(&mut de, &mut source);
            assert_eq!(&format!("{:?}", err.unwrap_err()), $err);
        }
    };
}

test_invalid!(
    test_invalid_string_bool,
    String,
    "true",
    r#"Error("invalid value: boolean `true`, expected file contents of variable `${my_test.file}` to be a string", line: 0, column: 0)"#
);
test_invalid!(
    test_invalid_string_signed,
    String,
    "-123",
    r#"Error("invalid value: integer `-123`, expected file contents of variable `${my_test.file}` to be a string", line: 0, column: 0)"#
);
test_invalid!(
    test_invalid_string_unsigned,
    String,
    "123",
    r#"Error("invalid value: integer `123`, expected file contents of variable `${my_test.file}` to be a string", line: 0, column: 0)"#
);
test_invalid!(
    test_invalid_string_float,
    String,
    "-123.0",
    r#"Error("invalid value: floating point `-123.0`, expected file contents of variable `${my_test.file}` to be a string", line: 0, column: 0)"#
);

macro_rules! test_any{
    ($name:ident, $value:literal, $($expected:tt)*) => {
        #[test]
        fn $name() {
            let tempdir = tempfile::tempdir().unwrap();
            std::fs::write(tempdir.path().join("my_test.file"), $value.to_string()).unwrap();

            let mut source = FileSource::new().with_base_path(tempdir.path());
            let mut de = serde_json::Deserializer::from_str("\"${my_test.file}\"");

            let v: serde_json::Value = serde_vars::deserialize(&mut de, &mut source).unwrap();
            assert_eq!(v, serde_json::json!($($expected)*));
        }
    };
}

test_any!(test_any_bool, true, true);
test_any!(test_any_integer, 123, 123);
test_any!(test_any_negative_integer, -123, -123);
test_any!(test_any_float, 123.45, 123.45);
test_any!(test_any_string, "foobar", "foobar");

// Tests for Source returning None when prefix/suffix don't match.
// These tests verify that non-variable strings pass through unchanged for string types,
// and error appropriately for primitive types.

macro_rules! test_no_variable_string {
    ($name:ident, $input:literal) => {
        #[test]
        fn $name() {
            let mut source = FileSource::new();
            let mut de = serde_json::Deserializer::from_str(concat!("\"", $input, "\""));

            let s: String = serde_vars::deserialize(&mut de, &mut source).unwrap();
            assert_eq!(s, $input);
        }
    };
}

// Strings without variable syntax should pass through unchanged
test_no_variable_string!(test_no_variable_plain_string, "hello world");
test_no_variable_string!(test_no_variable_partial_prefix, "${incomplete");
test_no_variable_string!(test_no_variable_partial_suffix, "incomplete}");
test_no_variable_string!(test_no_variable_wrong_prefix, "$[VAR]");
test_no_variable_string!(test_no_variable_empty, "");

macro_rules! test_no_variable_primitive_error {
    ($name:ident, $ty:ty, $input:literal, $expected:literal) => {
        #[test]
        fn $name() {
            let mut source = FileSource::new();
            let mut de = serde_json::Deserializer::from_str(concat!("\"", $input, "\""));

            let err: Result<$ty, _> = serde_vars::deserialize(&mut de, &mut source);
            assert_eq!(&format!("{:?}", err), $expected);
        }
    };
}

// Primitives with non-variable strings should error (one test per deserialize_* code path)
test_no_variable_primitive_error!(
    test_no_variable_bool_error,
    bool,
    "not_a_var",
    r#"Err(Error("invalid type: string \"not_a_var\", expected a boolean", line: 0, column: 0))"#
);
test_no_variable_primitive_error!(
    test_no_variable_i32_error,
    i32,
    "not_a_var",
    r#"Err(Error("invalid type: string \"not_a_var\", expected i32", line: 0, column: 0))"#
);
test_no_variable_primitive_error!(
    test_no_variable_f64_error,
    f64,
    "not_a_var",
    r#"Err(Error("invalid type: string \"not_a_var\", expected f64", line: 0, column: 0))"#
);

#[test]
fn test_no_variable_with_custom_prefix() {
    let tempdir = tempfile::tempdir().unwrap();
    std::fs::write(tempdir.path().join("secret.txt"), "secret_value").unwrap();

    // Source configured with custom prefix/suffix
    let mut source = FileSource::new()
        .with_base_path(tempdir.path())
        .with_variable_prefix("<<file:")
        .with_variable_suffix(">>");

    // This uses default ${} syntax which won't match the custom <<file:>> syntax
    let mut de = serde_json::Deserializer::from_str(r#""${secret.txt}""#);
    let s: String = serde_vars::deserialize(&mut de, &mut source).unwrap();
    // Should pass through unchanged since ${} doesn't match <<file:>>
    assert_eq!(s, "${secret.txt}");

    // This uses the correct syntax
    let mut de = serde_json::Deserializer::from_str(r#""<<file:secret.txt>>""#);
    let s: String = serde_vars::deserialize(&mut de, &mut source).unwrap();
    assert_eq!(s, "secret_value");
}
