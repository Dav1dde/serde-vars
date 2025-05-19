use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

use serde::Deserialize;
use serde_vars::MapSource;

macro_rules! test_lookup {
    ($name:ident, $value:literal, $ty:ty) => {
        test_lookup!($name, $value, $ty, $value);
    };
    ($name:ident, $value:literal, $ty:ty, $expected:literal) => {
        #[test]
        fn $name() {
            let mut source =
                MapSource::new(HashMap::from([("FOO".to_owned(), $value.to_string())]));
            let mut de = serde_json::Deserializer::from_str("\"${FOO}\"");

            let s: $ty = serde_vars::deserialize(&mut de, &mut source).unwrap();
            assert_eq!(s, $expected);
        }
    };
}

test_lookup!(test_lookup_string, "bAr", String);
test_lookup!(test_lookup_string_wrapped, r#""bAr""#, String, "bAr");
test_lookup!(test_lookup_string_int_wrapped, r#""123""#, String, "123");
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
            let mut source = MapSource::default();
            let mut de = serde_json::Deserializer::from_str("\"${DOES_NOT_EXIST}\"");

            let err: Result<$ty, _> = serde_vars::deserialize(&mut de, &mut source);
            insta::assert_debug_snapshot!(
                err,
                @r###"
            Err(
                Error("got variable `${DOES_NOT_EXIST}`, but it does not exist", line: 0, column: 0),
            )
            "###
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
    ($name:ident, $ty:ty, $value:literal, $($err:tt)*) => {
        #[test]
        fn $name() {
            let mut source =
                MapSource::new(HashMap::from([("MY_VAR".to_owned(), $value.to_string())]));
            let mut de = serde_json::Deserializer::from_str("\"${MY_VAR}\"");

            let err: Result<$ty, _> = serde_vars::deserialize(&mut de, &mut source);
            insta::assert_debug_snapshot!(err.unwrap_err(), @$($err)*);
        }
    };
}

test_invalid!(
    test_invalid_string_bool,
    String,
    "true",
    r#"Error("invalid value: boolean `true`, expected variable `${MY_VAR}` to be a string", line: 0, column: 0)"#
);
test_invalid!(
    test_invalid_string_signed,
    String,
    "-123",
    r#"Error("invalid value: integer `-123`, expected variable `${MY_VAR}` to be a string", line: 0, column: 0)"#
);
test_invalid!(
    test_invalid_string_unsigned,
    String,
    "123",
    r#"Error("invalid value: integer `123`, expected variable `${MY_VAR}` to be a string", line: 0, column: 0)"#
);
test_invalid!(
    test_invalid_string_float,
    String,
    "-123.0",
    r#"Error("invalid value: floating point `-123.0`, expected variable `${MY_VAR}` to be a string", line: 0, column: 0)"#
);

#[derive(Debug, Deserialize)]
#[expect(dead_code, reason = "debug impl is used to assert")]
struct Complex<'a> {
    bool: bool,

    i8: i8,
    i16: i16,
    i32: i32,
    i64: i64,

    u8: u8,
    u16: u16,
    u32: u32,
    u64: u64,

    f32: f32,
    f64: f64,

    #[serde(borrow)]
    str: Cow<'a, str>,
    string: String,
    byte_buf: serde_bytes::ByteBuf,

    map_u32: BTreeMap<String, u32>,
    vec_u32: Vec<u32>,
    opt_u32: Option<u32>,

    nested: Nested<'a>,
    r#enum: EnumAny,
    new_type: NewType,
    tuple: (String, u32),
}

#[derive(Debug, Deserialize)]
#[expect(dead_code, reason = "debug impl is used to assert")]
struct Nested<'a> {
    str: Cow<'a, str>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "v")]
#[expect(dead_code, reason = "debug impl is used to assert")]
enum EnumAny {
    VariantA { value: String },
    VariantB { value: u32 },
}

#[derive(Debug, Deserialize)]
#[expect(dead_code, reason = "debug impl is used to assert")]
struct NewType(String);

#[test]
fn test_complex_vars() {
    let mut source = MapSource::new(HashMap::from([
        ("bool".to_owned(), "true".to_owned()),
        ("i8".to_owned(), "-20".to_owned()),
        ("i16".to_owned(), "-200".to_owned()),
        ("i32".to_owned(), "42".to_owned()),
        ("i64".to_owned(), "1337".to_owned()),
        ("u8".to_owned(), "2".to_owned()),
        ("u16".to_owned(), "20".to_owned()),
        ("u32".to_owned(), "420".to_owned()),
        ("u64".to_owned(), "13370".to_owned()),
        ("f32".to_owned(), "1.0".to_owned()),
        ("f64".to_owned(), "2.0".to_owned()),
        ("str".to_owned(), "foo_str".to_owned()),
        ("string".to_owned(), "bar_string".to_owned()),
        ("byte_buf".to_owned(), "aaa".to_owned()),
        ("map_u32_0".to_owned(), "100".to_owned()),
        ("map_u32_1".to_owned(), "200".to_owned()),
        ("vec_u32_0".to_owned(), "3".to_owned()),
        ("vec_u32_1".to_owned(), "6".to_owned()),
        ("opt_u32".to_owned(), "42".to_owned()),
        ("nested_str".to_owned(), "meow".to_owned()),
        ("enum".to_owned(), "777".to_owned()),
        ("new_type".to_owned(), "not_so_new".to_owned()),
        ("tuple_string".to_owned(), "tuple".to_owned()),
        ("tuple_u32".to_owned(), "123".to_owned()),
    ]));
    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "bool": "${bool}",

        "i8": "${i8}",
        "i16": "${i16}",
        "i32": "${i32}",
        "i64": "${i64}",

        "u8": "${u8}",
        "u16": "${u16}",
        "u32": "${u32}",
        "u64": "${u64}",

        "f32": "${f32}",
        "f64": "${f64}",

        "str": "${str}",
        "string": "${string}",
        "byte_buf": "${byte_buf}",

        "map_u32": {"${KEY}": "${map_u32_0}", "other": "${map_u32_1}", "another": 42},
        "vec_u32": [0, 1, 2, "${vec_u32_0}", 4, 5, "${vec_u32_1}"],
        "opt_u32": "${opt_u32}",

        "nested": {"str": "${nested_str}"},
        "enum": {"v": "VariantB", "value": "${enum}"},
        "new_type": "${new_type}",
        "tuple": ["${tuple_string}", "${tuple_u32}"]
    }"#,
    );

    let r: Complex = serde_vars::deserialize(&mut de, &mut source).unwrap();
    insta::assert_debug_snapshot!(r, @r###"
    Complex {
        bool: true,
        i8: -20,
        i16: -200,
        i32: 42,
        i64: 1337,
        u8: 2,
        u16: 20,
        u32: 420,
        u64: 13370,
        f32: 1.0,
        f64: 2.0,
        str: "foo_str",
        string: "bar_string",
        byte_buf: [
            97,
            97,
            97,
        ],
        map_u32: {
            "${KEY}": 100,
            "another": 42,
            "other": 200,
        },
        vec_u32: [
            0,
            1,
            2,
            3,
            4,
            5,
            6,
        ],
        opt_u32: Some(
            42,
        ),
        nested: Nested {
            str: "meow",
        },
        enum: VariantB {
            value: 777,
        },
        new_type: NewType(
            "not_so_new",
        ),
        tuple: (
            "tuple",
            123,
        ),
    }
    "###);
}

#[test]
fn test_complex_no_vars() {
    let mut source = MapSource::default();
    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "bool": true,

        "i8": -20,
        "i16": -200,
        "i32": 42,
        "i64": 1337,

        "u8": 2,
        "u16": 20,
        "u32": 420,
        "u64": 13370,

        "f32": 1.0,
        "f64": 2.0,

        "str": "foo_str",
        "string": "bar_string",
        "byte_buf": "aaa",

        "map_u32": {"${KEY}": 100, "other": 200, "another": 42},
        "vec_u32": [0, 1, 2, 3, 4, 5, 6],
        "opt_u32": 42,

        "nested": {"str": "meow"},
        "enum": {"v": "VariantB", "value": 777},
        "new_type": "not_so_new",
        "tuple": ["tuple", 123]
    }"#,
    );

    let r: Complex = serde_vars::deserialize(&mut de, &mut source).unwrap();
    insta::assert_debug_snapshot!(r, @r###"
    Complex {
        bool: true,
        i8: -20,
        i16: -200,
        i32: 42,
        i64: 1337,
        u8: 2,
        u16: 20,
        u32: 420,
        u64: 13370,
        f32: 1.0,
        f64: 2.0,
        str: "foo_str",
        string: "bar_string",
        byte_buf: [
            97,
            97,
            97,
        ],
        map_u32: {
            "${KEY}": 100,
            "another": 42,
            "other": 200,
        },
        vec_u32: [
            0,
            1,
            2,
            3,
            4,
            5,
            6,
        ],
        opt_u32: Some(
            42,
        ),
        nested: Nested {
            str: "meow",
        },
        enum: VariantB {
            value: 777,
        },
        new_type: NewType(
            "not_so_new",
        ),
        tuple: (
            "tuple",
            123,
        ),
    }
    "###);
}

#[test]
fn test_enum_any_integer() {
    let mut source = MapSource::new(HashMap::from([("FOO".to_owned(), "123".to_owned())]));
    let mut de = serde_json::Deserializer::from_str(r#"{"v": "VariantB", "value": "${FOO}"}"#);

    let r: EnumAny = serde_vars::deserialize(&mut de, &mut source).unwrap();
    insta::assert_debug_snapshot!(r, @r###"
    VariantB {
        value: 123,
    }
    "###);
}

#[test]
fn test_enum_any_string() {
    let mut source = MapSource::new(HashMap::from([("FOO".to_owned(), "foobar".to_owned())]));
    let mut de = serde_json::Deserializer::from_str(r#"{"v": "VariantA", "value": "${FOO}"}"#);

    let r: EnumAny = serde_vars::deserialize(&mut de, &mut source).unwrap();
    insta::assert_debug_snapshot!(r, @r###"
    VariantA {
        value: "foobar",
    }
    "###);
}

#[test]
fn test_enum_any_string_num() {
    let mut source = MapSource::new(HashMap::from([("FOO".to_owned(), r#""123""#.to_owned())]));
    let mut de = serde_json::Deserializer::from_str(r#"{"v": "VariantA", "value": "${FOO}"}"#);

    let r: EnumAny = serde_vars::deserialize(&mut de, &mut source).unwrap();
    insta::assert_debug_snapshot!(r, @r###"
    VariantA {
        value: "123",
    }
    "###);
}
