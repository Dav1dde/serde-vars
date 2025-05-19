use std::{borrow::Cow, marker::PhantomData};

use serde::de::{self, Deserialize, Visitor};

use crate::{content::Content, source::Source};

/// A deserializer which substitutes strings with values provided from a [`Source`].
///
/// It allows strings in place of arbitrary primitive types to be resolved through the
/// provided [`Source`].
///
///
/// # Examples:
///
/// ```
/// # #[derive(PartialEq)]
/// #[derive(Debug, serde::Deserialize)]
/// struct Config {
///     redis: Redis,
/// }
///
/// # #[derive(PartialEq)]
/// #[derive(Debug, serde::Deserialize)]
/// struct Redis {
///     host: std::net::Ipv4Addr,
///     port: u16,
///     timeout: u32,
/// }
///
/// fn read_config() -> String {
///     return r#"{
///         "redis": {
///             "host": "${REDIS_HOST}",
///             "port": "${REDIS_PORT}",
///             "timeout": 5
///         }
///     }"#.to_owned();
/// }
///
/// let mut source = serde_vars::EnvSource::default();
/// # unsafe { std::env::set_var("REDIS_HOST", "127.0.0.1"); }
/// # unsafe { std::env::set_var("REDIS_PORT", "9977"); }
///
/// let config: Config = {
///     let config = read_config();
///
///     let mut de = serde_json::Deserializer::from_str(&config);
///     serde_vars::deserialize(&mut de, &mut source).unwrap()
/// };
///
/// assert_eq!(config, Config {
///     redis: Redis {
///         host: [127, 0, 0, 1].into(),
///         port: 9977,
///         timeout: 5
///     }
/// });
/// ```
pub struct Deserializer<'a, D, S> {
    de: D,
    source: &'a mut S,
}

impl<'a, D, S> Deserializer<'a, D, S> {
    pub fn new(de: D, source: &'a mut S) -> Self {
        Self { de, source }
    }
}

impl<'de, D, S> de::Deserializer<'de> for Deserializer<'_, D, S>
where
    D: de::Deserializer<'de>,
    S: Source,
{
    type Error = D::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_any(Wrap::new(visitor, self.source))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_bool(visitor)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_i8(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_i16(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_i32(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_i64(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_u8(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_u16(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_u32(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_u64(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_f32(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_f64(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_char(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // TODO: support zero copy/borrowed strings here.
        // To support this we need a custom visitor which can differentiate between
        // a borrowed `&'de str` and just a referenced `&str` as well as accept `String`.
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // Directly deserialize into a `String` here, because we do not need support arbitrary
        // types here (through any). Here we can always expect a string, no matter if the value
        // contains a variable reference or not.
        // This allows formats, like YAML, which can deserialize a value into multiple types,
        // to yield a string when they otherwise would yield another type (e.g. u64).
        let content = Content::String(Deserialize::deserialize(self.de)?);
        ContentVarDeserializer::new(content, self.source).deserialize_string(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // TODO: support zero copy/borrowed bytes here.
        // To support this we need a custom visitor which can differentiate between
        // a borrowed `&'de str` and just a referenced `&str` as well as accept `String`.
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // See `deserialize_string` why we deserialize into a byte buf directly here.
        let content = Content::ByteBuf(crate::value::deserialize_byte_buf(self.de)?);
        ContentVarDeserializer::new(content, self.source).deserialize_byte_buf(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_option(Wrap::new(visitor, self.source))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        ContentVarDeserializer::from_de(self.de, self.source)?.deserialize_unit(visitor)
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de
            .deserialize_unit_struct(name, Wrap::new(visitor, self.source))
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de
            .deserialize_newtype_struct(name, Wrap::new(visitor, self.source))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_seq(Wrap::new(visitor, self.source))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de
            .deserialize_tuple(len, Wrap::new(visitor, self.source))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de
            .deserialize_tuple_struct(name, len, Wrap::new(visitor, self.source))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_map(Wrap::new(visitor, self.source))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de
            .deserialize_struct(name, fields, Wrap::new(visitor, self.source))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de
            .deserialize_enum(name, variants, Wrap::new(visitor, self.source))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de
            .deserialize_identifier(Wrap::new(visitor, self.source))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.de
            .deserialize_ignored_any(Wrap::new(visitor, self.source))
    }
}

struct Wrap<'a, T, S> {
    delegate: T,
    source: &'a mut S,
}

impl<'a, T, S> Wrap<'a, T, S> {
    fn new(delegate: T, source: &'a mut S) -> Self {
        Self { delegate, source }
    }
}

/// This implementation is only called for non-primitive types, like
/// struct, enum, option etc. in which case it forwards nested
/// [`de::Deserializer`] calls to the original [`Deserializer`].
///
/// And it is also called as a dispatch for [`de::Deserializer::deserialize_any`],
/// any primitive callback like, `visit_bool` can be expected to be called
/// from a code path which invoked `deserialize_any`.
/// When a string variant is invoked through such a code path, the visitor
/// delegates it to [`Source::expand_any`].
impl<'de, T, S> Visitor<'de> for Wrap<'_, T, S>
where
    T: Visitor<'de>,
    S: Source,
{
    type Value = T::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.delegate.expecting(formatter)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_bool(v)
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i8(v)
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i16(v)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i32(v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i64(v)
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_i128(v)
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u8(v)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u16(v)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u32(v)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u64(v)
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_u128(v)
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_f32(v)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_f64(v)
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_char(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.source
            .expand_any(Cow::Borrowed(v))?
            .visit(self.delegate)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.source
            .expand_any(Cow::Borrowed(v))?
            .visit_borrowed(self.delegate)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.source.expand_any(Cow::Owned(v))?.visit(self.delegate)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_bytes(v)
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_borrowed_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_byte_buf(v)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.delegate
            .visit_some(Deserializer::new(deserializer, self.source))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.delegate.visit_unit()
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.delegate
            .visit_newtype_struct(Deserializer::new(deserializer, self.source))
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        self.delegate.visit_seq(Wrap::new(seq, self.source))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        self.delegate.visit_map(Wrap::new(map, self.source))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        self.delegate.visit_enum(Wrap::new(data, self.source))
    }
}

impl<'de, T, S> de::MapAccess<'de> for Wrap<'_, T, S>
where
    S: Source,
    T: de::MapAccess<'de>,
{
    type Error = T::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        // Do not wrap the key, we do not want to resolve keys.
        self.delegate.next_key_seed(seed)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.delegate.next_value_seed(Wrap::new(seed, self.source))
    }
}

impl<'de, T, S> de::SeqAccess<'de> for Wrap<'_, T, S>
where
    S: Source,
    T: de::SeqAccess<'de>,
{
    type Error = T::Error;

    fn next_element_seed<Seed>(&mut self, seed: Seed) -> Result<Option<Seed::Value>, Self::Error>
    where
        Seed: de::DeserializeSeed<'de>,
    {
        self.delegate
            .next_element_seed(Wrap::new(seed, self.source))
    }
}

impl<'de, T, S> de::EnumAccess<'de> for Wrap<'_, T, S>
where
    T: de::EnumAccess<'de>,
    S: Source,
{
    type Error = T::Error;
    type Variant = T::Variant;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.delegate.variant_seed(Wrap::new(seed, self.source))
    }
}

impl<'de, T, S> de::DeserializeSeed<'de> for Wrap<'_, T, S>
where
    T: de::DeserializeSeed<'de>,
    S: Source,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let deserializer = Deserializer::new(deserializer, self.source);
        T::deserialize(self.delegate, deserializer)
    }
}

/// A [`de::Deserializer`] holding a [`Content`] that expands strings using a [`Source`].
struct ContentVarDeserializer<'a, 'de, E, S> {
    content: Content<'de>,
    err: PhantomData<E>,
    source: &'a mut S,
}

impl<'a, 'de, E, S> ContentVarDeserializer<'a, 'de, E, S> {
    fn new(content: Content<'de>, source: &'a mut S) -> Self {
        Self {
            content,
            err: PhantomData,
            source,
        }
    }

    fn from_de<D>(deserializer: D, source: &'a mut S) -> Result<Self, E>
    where
        D: de::Deserializer<'de, Error = E>,
    {
        Content::deserialize(deserializer).map(|content| Self::new(content, source))
    }
}

impl<'de, E, S> ContentVarDeserializer<'_, 'de, E, S>
where
    E: de::Error,
    S: Source,
{
    #[cold]
    fn invalid_type(self, exp: &dyn de::Expected) -> E {
        de::Error::invalid_type(self.content.unexpected(), exp)
    }

    fn deserialize_integer<V, F>(
        self,
        visitor: V,
        f: impl FnOnce(V, F) -> Result<V::Value, E>,
        mut conv: impl FnMut(&mut S, &str) -> Result<F, E>,
    ) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        match self.content {
            Content::U8(v) => visitor.visit_u8(v),
            Content::U16(v) => visitor.visit_u16(v),
            Content::U32(v) => visitor.visit_u32(v),
            Content::U64(v) => visitor.visit_u64(v),
            Content::I8(v) => visitor.visit_i8(v),
            Content::I16(v) => visitor.visit_i16(v),
            Content::I32(v) => visitor.visit_i32(v),
            Content::I64(v) => visitor.visit_i64(v),
            Content::Str(s) => f(visitor, conv(self.source, s)?),
            Content::String(ref s) => f(visitor, conv(self.source, s)?),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_float<V, F>(
        self,
        visitor: V,
        f: impl FnOnce(V, F) -> Result<V::Value, E>,
        mut conv: impl FnMut(&mut S, &str) -> Result<F, E>,
    ) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        match self.content {
            Content::F32(v) => visitor.visit_f32(v),
            Content::F64(v) => visitor.visit_f64(v),
            Content::U8(v) => visitor.visit_u8(v),
            Content::U16(v) => visitor.visit_u16(v),
            Content::U32(v) => visitor.visit_u32(v),
            Content::U64(v) => visitor.visit_u64(v),
            Content::I8(v) => visitor.visit_i8(v),
            Content::I16(v) => visitor.visit_i16(v),
            Content::I32(v) => visitor.visit_i32(v),
            Content::I64(v) => visitor.visit_i64(v),
            Content::Str(s) => f(visitor, conv(self.source, s)?),
            Content::String(ref s) => f(visitor, conv(self.source, s)?),
            _ => Err(self.invalid_type(&visitor)),
        }
    }
}

impl<'de, E, S> de::Deserializer<'de> for ContentVarDeserializer<'_, 'de, E, S>
where
    E: de::Error,
    S: Source,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.content {
            Content::Bool(v) => visitor.visit_bool(v),
            Content::Str(s) => visitor.visit_bool(self.source.expand_bool(s)?),
            Content::String(ref s) => visitor.visit_bool(self.source.expand_bool(s)?),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor, Visitor::visit_i8, Source::expand_i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor, Visitor::visit_i16, Source::expand_i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor, Visitor::visit_i32, Source::expand_i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor, Visitor::visit_i64, Source::expand_i64)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor, Visitor::visit_u8, Source::expand_u8)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor, Visitor::visit_u16, Source::expand_u16)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor, Visitor::visit_u32, Source::expand_u32)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor, Visitor::visit_u64, Source::expand_u64)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_float(visitor, Visitor::visit_f32, Source::expand_f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_float(visitor, Visitor::visit_f64, Source::expand_f64)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.content {
            Content::String(_) | Content::Str(_) => self.deserialize_str(visitor),
            Content::Char(v) => visitor.visit_char(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match match self.content {
            Content::String(v) => self.source.expand_str(Cow::Owned(v))?,
            Content::Str(v) => self.source.expand_str(Cow::Borrowed(v))?,
            _ => return Err(self.invalid_type(&visitor)),
        } {
            Cow::Owned(s) => visitor.visit_string(s),
            Cow::Borrowed(s) => visitor.visit_borrowed_str(s),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match match self.content {
            Content::String(_) | Content::Str(_) => return self.deserialize_str(visitor),
            Content::ByteBuf(v) => self.source.expand_bytes(Cow::Owned(v))?,
            Content::Bytes(v) => self.source.expand_bytes(Cow::Borrowed(v))?,
            _ => return Err(self.invalid_type(&visitor)),
        } {
            Cow::Owned(v) => visitor.visit_byte_buf(v),
            Cow::Borrowed(v) => visitor.visit_bytes(v),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}
