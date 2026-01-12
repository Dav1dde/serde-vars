//! Dynamic variable expansion.

use std::borrow::Cow;

use serde::de;

mod file;
mod string;
mod utils;

pub use self::file::*;
pub use self::string::*;

/// A [`Source`] expands a variable string into a concrete value.
///
/// All methods return `Result<Option<T>, E>` where:
/// - `Ok(None)` means the input string is not recognized as a variable (e.g., prefix/suffix don't match)
/// - `Ok(Some(v))` means the variable was successfully resolved
/// - `Err(e)` means an error occurred (e.g., variable recognized but resolution failed)
pub trait Source {
    /// Expands a variable string to a boolean.
    fn expand_bool<E>(&mut self, v: &str) -> Result<Option<bool>, E>
    where
        E: de::Error;

    /// Expands a variable string to an `i8`.
    fn expand_i8<E>(&mut self, v: &str) -> Result<Option<i8>, E>
    where
        E: de::Error;

    /// Expands a variable string to an `i16`.
    fn expand_i16<E>(&mut self, v: &str) -> Result<Option<i16>, E>
    where
        E: de::Error;

    /// Expands a variable string to an `i32`.
    fn expand_i32<E>(&mut self, v: &str) -> Result<Option<i32>, E>
    where
        E: de::Error;

    /// Expands a variable string to an `i64`.
    fn expand_i64<E>(&mut self, v: &str) -> Result<Option<i64>, E>
    where
        E: de::Error;

    /// Expands a variable string to an `u8`.
    fn expand_u8<E>(&mut self, v: &str) -> Result<Option<u8>, E>
    where
        E: de::Error;

    /// Expands a variable string to an `u16`.
    fn expand_u16<E>(&mut self, v: &str) -> Result<Option<u16>, E>
    where
        E: de::Error;

    /// Expands a variable string to an `u32`.
    fn expand_u32<E>(&mut self, v: &str) -> Result<Option<u32>, E>
    where
        E: de::Error;

    /// Expands a variable string to an `u64`.
    fn expand_u64<E>(&mut self, v: &str) -> Result<Option<u64>, E>
    where
        E: de::Error;

    /// Expands a variable string to a `f32`.
    fn expand_f32<E>(&mut self, v: &str) -> Result<Option<f32>, E>
    where
        E: de::Error;

    /// Expands a variable string to a `f64`.
    fn expand_f64<E>(&mut self, v: &str) -> Result<Option<f64>, E>
    where
        E: de::Error;

    /// Expands a variable string to string.
    ///
    /// Returns `Ok(None)` if the string does not contain a variable reference.
    /// The caller should use the original string in this case.
    fn expand_str<'a, E>(&mut self, v: &'a str) -> Result<Option<Cow<'a, str>>, E>
    where
        E: de::Error;

    /// Expands bytes into other bytes.
    ///
    /// Returns `Ok(None)` if the bytes do not contain a variable reference.
    /// The caller should use the original bytes in this case.
    ///
    /// Implementations which can expand strings, should also expand byte sequences
    /// which are valid utf-8.
    fn expand_bytes<'a, E>(&mut self, v: &'a [u8]) -> Result<Option<Cow<'a, [u8]>>, E>
    where
        E: de::Error;

    /// Expands a variable string to [`Any`].
    ///
    /// Required for self-describing deserialization, where the resulting type
    /// depends on the type deserialized.
    ///
    /// Returns `Ok(None)` if the string does not contain a variable reference.
    /// The caller should use the original string in this case.
    fn expand_any<'a, E>(&mut self, v: &'a str) -> Result<Option<Any<'a>>, E>
    where
        E: de::Error;
}

/// Type returned by [`Source::expand_any`].
///
/// Represents any primitive type that can be parsed by a [`Source`].
pub enum Any<'a> {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Str(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
}

impl<'a> Any<'a> {
    /// Turns this [`Self`] into a [`de::Unexpected`] for error handling.
    pub fn unexpected(&self) -> de::Unexpected<'_> {
        match self {
            Any::Bool(v) => de::Unexpected::Bool(*v),
            Any::I8(v) => de::Unexpected::Signed(i64::from(*v)),
            Any::I16(v) => de::Unexpected::Signed(i64::from(*v)),
            Any::I32(v) => de::Unexpected::Signed(i64::from(*v)),
            Any::I64(v) => de::Unexpected::Signed(*v),
            Any::U8(v) => de::Unexpected::Unsigned(u64::from(*v)),
            Any::U16(v) => de::Unexpected::Unsigned(u64::from(*v)),
            Any::U32(v) => de::Unexpected::Unsigned(u64::from(*v)),
            Any::U64(v) => de::Unexpected::Unsigned(*v),
            Any::F32(v) => de::Unexpected::Float(f64::from(*v)),
            Any::F64(v) => de::Unexpected::Float(*v),
            Any::Str(v) => de::Unexpected::Str(v),
            Any::Bytes(v) => de::Unexpected::Bytes(v),
        }
    }

    pub(crate) fn visit_borrowed<V, E>(self, visitor: V) -> Result<V::Value, E>
    where
        V: de::Visitor<'a>,
        E: de::Error,
    {
        match self {
            Any::Str(Cow::Borrowed(v)) => visitor.visit_borrowed_str(v),
            Any::Bytes(Cow::Borrowed(v)) => visitor.visit_borrowed_bytes(v),
            other => other.visit(visitor),
        }
    }

    pub(crate) fn visit<'de, V, E>(self, visitor: V) -> Result<V::Value, E>
    where
        V: de::Visitor<'de>,
        E: de::Error,
    {
        match self {
            Any::Bool(v) => visitor.visit_bool(v),
            Any::I8(v) => visitor.visit_i8(v),
            Any::I16(v) => visitor.visit_i16(v),
            Any::I32(v) => visitor.visit_i32(v),
            Any::I64(v) => visitor.visit_i64(v),
            Any::U8(v) => visitor.visit_u8(v),
            Any::U16(v) => visitor.visit_u16(v),
            Any::U32(v) => visitor.visit_u32(v),
            Any::U64(v) => visitor.visit_u64(v),
            Any::F32(v) => visitor.visit_f32(v),
            Any::F64(v) => visitor.visit_f64(v),
            Any::Str(Cow::Owned(v)) => visitor.visit_string(v),
            Any::Str(Cow::Borrowed(v)) => visitor.visit_str(v),
            Any::Bytes(Cow::Owned(v)) => visitor.visit_byte_buf(v),
            Any::Bytes(Cow::Borrowed(v)) => visitor.visit_bytes(v),
        }
    }
}
