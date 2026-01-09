use std::marker::PhantomData;

use serde::de;

pub enum Content<'de> {
    Bool(bool),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

    F32(f32),
    F64(f64),

    Char(char),
    String(String),
    Str(&'de str),
    ByteBuf(Vec<u8>),
    Bytes(&'de [u8]),
}

impl Content<'_> {
    #[cold]
    pub fn unexpected(&self) -> de::Unexpected<'_> {
        match *self {
            Content::Bool(b) => de::Unexpected::Bool(b),
            Content::U8(n) => de::Unexpected::Unsigned(n as u64),
            Content::U16(n) => de::Unexpected::Unsigned(n as u64),
            Content::U32(n) => de::Unexpected::Unsigned(n as u64),
            Content::U64(n) => de::Unexpected::Unsigned(n),
            Content::I8(n) => de::Unexpected::Signed(n as i64),
            Content::I16(n) => de::Unexpected::Signed(n as i64),
            Content::I32(n) => de::Unexpected::Signed(n as i64),
            Content::I64(n) => de::Unexpected::Signed(n),
            Content::F32(f) => de::Unexpected::Float(f as f64),
            Content::F64(f) => de::Unexpected::Float(f),
            Content::Char(c) => de::Unexpected::Char(c),
            Content::String(ref s) => de::Unexpected::Str(s),
            Content::Str(s) => de::Unexpected::Str(s),
            Content::ByteBuf(ref b) => de::Unexpected::Bytes(b),
            Content::Bytes(b) => de::Unexpected::Bytes(b),
        }
    }
}

impl<'de> de::Deserialize<'de> for Content<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(ContentVisitor::new())
    }
}

pub struct ContentVisitor<'de> {
    value: PhantomData<Content<'de>>,
}

impl ContentVisitor<'_> {
    pub fn new() -> Self {
        Self { value: PhantomData }
    }
}

impl<'de> de::Visitor<'de> for ContentVisitor<'de> {
    type Value = Content<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any primitive value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::Bool(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::I8(v))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::I16(v))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::I32(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::I64(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::U8(v))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::U16(v))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::U32(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::U64(v))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::F32(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::F64(v))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::Char(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::String(v.to_owned()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::Str(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::String(v))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::ByteBuf(v.to_vec()))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::Bytes(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Content::ByteBuf(v))
    }
}
