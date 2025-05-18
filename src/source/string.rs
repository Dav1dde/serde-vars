use std::{borrow::Cow, collections::HashMap};

use super::{Any, Source};
use serde::de::{self, Unexpected};

/// A simple lookup function, used by the [`StringSource`].
pub trait StringLookup {
    /// Looks up the variable `v` and returns its value.
    ///
    /// Returns `None` if the variable cannot be found.
    fn lookup(&mut self, v: &str) -> Option<String>;
}

/// A [`StringLookup`] which uses the process environment.
///
/// Generally used through [`EnvSource`].
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvLookup;

impl StringLookup for EnvLookup {
    fn lookup(&mut self, v: &str) -> Option<String> {
        std::env::var(v).ok()
    }
}

impl StringLookup for HashMap<String, String> {
    fn lookup(&mut self, v: &str) -> Option<String> {
        self.get(v).cloned()
    }
}

/// A source which uses values from the environment.
///
/// See the [`crate`] and [`StringSource`] documentation for more details.
///
/// # Examples:
///
/// ```
/// use serde_vars::EnvSource;
///
/// let mut source = EnvSource::default();
/// # unsafe { std::env::set_var("MY_VAR", "some secret value"); }
///
/// let mut de = serde_json::Deserializer::from_str(r#""${MY_VAR}""#);
/// let r: String = serde_vars::deserialize(&mut de, &mut source).unwrap();
/// assert_eq!(r, "some secret value");
/// ```
pub type EnvSource = StringSource<EnvLookup>;
/// A source which uses values provided from a [`HashMap`].
///
/// See the [`crate`] and [`StringSource`] documentation for more details.
pub type MapSource = StringSource<HashMap<String, String>>;

/// A [`Source`] which provides values using a string based [`StringLookup`].
///
/// This source only works with strings, but since data can be serialized into any type,
/// it implements some basic conversion of data types through [`std::str::FromStr`].
///
/// During deserialization most of the time, the target type is known. For example, when
/// deserializing a field `foo: u32`, the target type is known to be `u32`. In these cases the [`StringSource`],
/// will attempt to parse the requested type from the provided string.
///
/// When deserializing self-describing formats, like JSON or YAML into dynamic containers,
/// like for example:
///
/// ```
/// #[derive(serde::Deserialize)]
/// #[serde(untagged)]
/// enum StringOrInt {
///     String(String),
///     Int(u64),
/// }
/// ```
///
/// The target type is inferred from the parsed type. In these cases the [`StringSource`],
/// needs to make the type decision.
///
/// Dynamic parsing ([`Source::expand_any`]), parses in order `bool`, `u64`, `f64`, `String` and yields the
/// first one which succeeds. To explicitly force a string, the source allows to explicitly wrap
/// the value in an additional pair of `"`, which will be stripped:
///
/// - `true`, `false` -> `bool`
/// - `123`, `42` -> `u64`
/// - `-123`, `-42` -> `i64`
/// - `-123.0`, `42.12` -> `f64`
/// - `foobar`, `"foobar"`, `"true"`, `123 some more` -> `String`
///
/// For consistency reasons, known string expansions use the same parsing logic and require
/// ambiguous values to be explicitly marked as a string.
#[derive(Debug)]
pub struct StringSource<T> {
    prefix: String,
    suffix: String,
    lookup: T,
}

impl<T> StringSource<T> {
    /// Creates a [`Self`] using the specified [`StringLookup`].
    ///
    /// By default the created source uses `${` and `}` as variable specifiers.
    /// These can be changed using [`Self::with_variable_prefix`] and [`Self::with_variable_suffix`].
    ///
    /// # Examples:
    ///
    /// ```
    /// use serde_vars::StringSource;
    /// use std::collections::HashMap;
    ///
    /// let source = HashMap::from([("MY_VAR".to_owned(), "some secret value".to_owned())]);
    /// let mut source = StringSource::new(source);
    ///
    /// let mut de = serde_json::Deserializer::from_str(r#""${MY_VAR}""#);
    /// let r: String = serde_vars::deserialize(&mut de, &mut source).unwrap();
    /// assert_eq!(r, "some secret value");
    /// ```
    pub fn new(lookup: T) -> Self {
        Self {
            prefix: "${".to_owned(),
            suffix: "}".to_owned(),
            lookup,
        }
    }

    /// Changes the variable prefix.
    ///
    /// # Examples:
    ///
    /// ```
    /// # use serde_vars::StringSource;
    /// # use std::collections::HashMap;
    /// #
    /// # let source = HashMap::from([("MY_VAR".to_owned(), "some secret value".to_owned())]);
    /// # let mut source = StringSource::new(source).with_variable_prefix("$").with_variable_suffix("");
    /// #
    /// let mut de = serde_json::Deserializer::from_str(r#""$MY_VAR""#);
    /// let r: String = serde_vars::deserialize(&mut de, &mut source).unwrap();
    /// assert_eq!(r, "some secret value");
    /// ```
    pub fn with_variable_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Changes the variable suffix.
    pub fn with_variable_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    /// Returns the contained [`StringLookup`].
    pub fn into_inner(self) -> T {
        self.lookup
    }
}

impl<T> Default for StringSource<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> StringSource<T>
where
    T: StringLookup,
{
    fn missing_variable<E>(&self, var: &str) -> E
    where
        E: de::Error,
    {
        E::custom(format!(
            "got variable `{}{var}{}`, but it does not exist",
            self.prefix, self.suffix
        ))
    }

    fn expected_variable<E>(&self, v: &str, expected: &str) -> E
    where
        E: de::Error,
    {
        E::invalid_value(
            de::Unexpected::Str(v),
            &format!("expected {expected} or a variable").as_str(),
        )
    }

    fn mismatched_type<E>(&self, var: &str, unexpected: Unexpected<'_>, expected: &str) -> E
    where
        E: de::Error,
    {
        E::invalid_value(
            unexpected,
            &format!(
                "variable `{}{var}{}` to be {expected}",
                self.prefix, self.suffix
            )
            .as_str(),
        )
    }

    fn parse_var<'a>(&mut self, v: &'a str) -> Option<&'a str> {
        v.strip_prefix(&self.prefix)?.strip_suffix(&self.suffix)
    }

    fn parsed<V, E>(&mut self, v: &str, expected: &str) -> Result<V, E>
    where
        V: std::str::FromStr,
        V::Err: std::fmt::Display,
        E: de::Error,
    {
        let Some(var) = self.parse_var(v) else {
            return Err(self.expected_variable(v, expected));
        };

        match self.lookup.lookup(var) {
            Some(value) => value
                .parse()
                .map_err(|_| self.mismatched_type(var, de::Unexpected::Str(&value), expected)),
            None => Err(self.missing_variable(var)),
        }
    }
}

impl<T> Source for StringSource<T>
where
    T: StringLookup,
{
    fn expand_str<'a, E>(&mut self, v: Cow<'a, str>) -> Result<Cow<'a, str>, E>
    where
        E: de::Error,
    {
        let Some(var) = self.parse_var(&v) else {
            // There is no variable in the string, the expanded variant is just the original.
            return Ok(v);
        };

        match self.lookup.lookup(var) {
            Some(value) => match parse(Cow::Owned(value)) {
                Any::Str(value) => Ok(value),
                other => Err(self.mismatched_type(var, other.unexpected(), "a string")),
            },
            None => Err(self.missing_variable(var)),
        }
    }

    fn expand_bool<E>(&mut self, v: &str) -> Result<bool, E>
    where
        E: de::Error,
    {
        self.parsed(v, "a boolean")
    }

    fn expand_i8<E>(&mut self, v: &str) -> Result<i8, E>
    where
        E: de::Error,
    {
        self.parsed(v, "a signed integer (i8)")
    }

    fn expand_i16<E>(&mut self, v: &str) -> Result<i16, E>
    where
        E: de::Error,
    {
        self.parsed(v, "a signed integer (i16)")
    }

    fn expand_i32<E>(&mut self, v: &str) -> Result<i32, E>
    where
        E: de::Error,
    {
        self.parsed(v, "a signed integer (i32)")
    }

    fn expand_i64<E>(&mut self, v: &str) -> Result<i64, E>
    where
        E: de::Error,
    {
        self.parsed(v, "a signed integer (i64)")
    }

    fn expand_u8<E>(&mut self, v: &str) -> Result<u8, E>
    where
        E: de::Error,
    {
        self.parsed(v, "an unsigned integer (i8)")
    }

    fn expand_u16<E>(&mut self, v: &str) -> Result<u16, E>
    where
        E: de::Error,
    {
        self.parsed(v, "an unsigned integer (i16)")
    }

    fn expand_u32<E>(&mut self, v: &str) -> Result<u32, E>
    where
        E: de::Error,
    {
        self.parsed(v, "an unsigned integer (i32)")
    }

    fn expand_u64<E>(&mut self, v: &str) -> Result<u64, E>
    where
        E: de::Error,
    {
        self.parsed(v, "an unsigned integer (i64)")
    }

    fn expand_f32<E>(&mut self, v: &str) -> Result<f32, E>
    where
        E: de::Error,
    {
        self.parsed(v, "a floating point")
    }

    fn expand_f64<E>(&mut self, v: &str) -> Result<f64, E>
    where
        E: de::Error,
    {
        self.parsed(v, "a floating point")
    }

    fn expand_any<'a, E>(&mut self, v: Cow<'a, str>) -> Result<Any<'a>, E>
    where
        E: de::Error,
    {
        let Some(var) = self.parse_var(&v) else {
            // There is no variable in the string, the expanded variant is just the original.
            return Ok(Any::Str(v));
        };

        self.lookup
            .lookup(var)
            .map(|value| parse(Cow::Owned(value)))
            .ok_or_else(|| self.missing_variable(var))
    }
}

fn strip_str(s: Cow<'_, str>) -> Cow<'_, str> {
    match s.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        Some(s) => Cow::Owned(s.to_owned()),
        None => s,
    }
}

fn parse(s: Cow<'_, str>) -> Any<'_> {
    match s.as_ref() {
        "true" => Any::Bool(true),
        "false" => Any::Bool(false),
        // Try in order:
        //  - parse f64
        //  - parse u64
        //  - parse string escape `"<str>"`
        //  - use the literal string
        v => v
            .parse()
            .map(Any::U64)
            .or_else(|_| v.parse().map(Any::I64))
            .or_else(|_| v.parse().map(Any::F64))
            .unwrap_or_else(|_| Any::Str(strip_str(s))),
    }
}
