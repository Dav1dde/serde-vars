use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use serde::de;

use crate::source::{utils, Any, Source};

// Possible future improvements:
//  - A file-system abstraction
//  - Abstract into a byte-source
//  - Allow modifications to conversions
//  - More validations (e.g. base-path)
//  - A way to specify base path for relative paths

/// A [`Source`] which provides values by reading them from the filesystem.
///
/// For string and byte types, the source will simply attempt to open the file and load its
/// contents.
///
/// If, during de-serialization, the target type is known, the source will attempt to load the file
/// as a string parse the value into the target type using [`std::str::FromStr`].
///
/// When de-serializing self-describing formats, like JSON or YAML into dynamic containers,
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
/// The target type is inferred from the loaded file contents. The source parses the file contents
/// in following order:
///
/// - `true`, `false` -> `bool`
/// - `123`, `42` -> `u64`
/// - `-123`, `-42` -> `i64`
/// - `-123.0`, `42.12` -> `f64`
/// - any valid UTF-8 string -> `String`
/// - -> `Vec<u8>`
///
/// # Warning:
///
/// This source must not be used with untrusted user input, it provides unfiltered access to the
/// filesystem.
pub struct FileSource {
    base_path: PathBuf,
    variable: utils::Variable,
}

impl FileSource {
    /// Creates a [`FileSource`].
    ///
    /// By default the created source uses `${` and `}` as variable specifiers.
    /// These can be changed using [`Self::with_variable_prefix`] and [`Self::with_variable_suffix`].
    ///
    /// # Examples:
    ///
    /// ```
    /// # let temp = tempfile::tempdir().unwrap();
    /// # std::fs::write(temp.path().join("my_file.txt"), "some secret value").unwrap();
    /// #
    /// use serde_vars::FileSource;
    ///
    /// let mut source = FileSource::new();
    /// # let mut source = source.with_base_path(temp.path());
    ///
    /// let mut de = serde_json::Deserializer::from_str(r#""${my_file.txt}""#);
    /// let r: String = serde_vars::deserialize(&mut de, &mut source).unwrap();
    /// assert_eq!(r, "some secret value");
    /// ```
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::new(),
            variable: Default::default(),
        }
    }

    /// Configures the base path to use for relative paths.
    ///
    /// The configured path is joined with relative paths. To be independent of the
    /// current working directory it is recommended to configure an absolute path.
    ///
    /// Note: There is no validation that a final path must be within that base directory.
    pub fn with_base_path<P>(mut self, path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        self.base_path = path.into();
        self
    }

    /// Changes the variable prefix.
    ///
    /// # Examples:
    ///
    /// ```
    /// # let temp = tempfile::tempdir().unwrap();
    /// # std::fs::write(temp.path().join("my_file.txt"), "some secret value").unwrap();
    /// #
    /// use serde_vars::FileSource;
    ///
    /// let mut source = FileSource::new().with_variable_prefix("${file:");
    /// # let mut source = source.with_base_path(temp.path());
    ///
    /// let mut de = serde_json::Deserializer::from_str(r#""${file:my_file.txt}""#);
    /// let r: String = serde_vars::deserialize(&mut de, &mut source).unwrap();
    /// assert_eq!(r, "some secret value");
    /// ```
    pub fn with_variable_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.variable.prefix = prefix.into();
        self
    }

    /// Changes the variable suffix.
    pub fn with_variable_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.variable.suffix = suffix.into();
        self
    }
}

impl FileSource {
    fn resolve_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        match path.is_absolute() {
            true => Cow::Borrowed(path),
            false => Cow::Owned(self.base_path.join(path)),
        }
    }

    fn io_error<E>(&self, path: &Path, v: &Path, error: std::io::Error) -> E
    where
        E: de::Error,
    {
        let path = path.display();
        let var = self.variable.fmt(v.display());
        E::custom(format!(
            "failed to read file `{path}` from variable `{var}`: {error}"
        ))
    }

    fn expected_variable<E>(&self, v: &str, expected: &str) -> E
    where
        E: de::Error,
    {
        let var = self.variable.fmt("<var>");
        E::invalid_value(
            de::Unexpected::Str(v),
            &format!("expected {expected} or a file variable `{var}`").as_str(),
        )
    }

    fn mismatched_type<E>(&self, var: &str, unexpected: de::Unexpected<'_>, expected: &str) -> E
    where
        E: de::Error,
    {
        let var = self.variable.fmt(var);
        E::invalid_value(
            unexpected,
            &format!("file contents of variable `{var}` to be {expected}").as_str(),
        )
    }

    fn parsed<V, E>(&mut self, v: &str, expected: &str) -> Result<V, E>
    where
        V: std::str::FromStr,
        V::Err: std::fmt::Display,
        E: de::Error,
    {
        let Some(var) = self.variable.parse_str(v) else {
            return Err(self.expected_variable(v, expected));
        };

        let path = self.resolve_path(var.as_ref());
        let value = std::fs::read_to_string(&path)
            .map_err(|error| self.io_error(&path, var.as_ref(), error))?;

        value
            .parse()
            .map_err(|_| self.mismatched_type(var, de::Unexpected::Str(&value), expected))
    }
}

impl Source for FileSource {
    fn expand_str<'a, E>(&mut self, v: Cow<'a, str>) -> Result<Cow<'a, str>, E>
    where
        E: serde::de::Error,
    {
        let Some(var) = self.variable.parse_str(&v) else {
            return Ok(v);
        };

        let path = self.resolve_path(var.as_ref());
        let value = std::fs::read_to_string(&path)
            .map_err(|error| self.io_error(&path, var.as_ref(), error))?;

        match utils::parse(Cow::Owned(value)) {
            Any::Str(value) => Ok(value),
            other => Err(self.mismatched_type(var, other.unexpected(), "a string")),
        }
    }

    fn expand_bytes<'a, E>(&mut self, v: Cow<'a, [u8]>) -> Result<Cow<'a, [u8]>, E>
    where
        E: serde::de::Error,
    {
        let Some(var) = self.variable.parse_bytes(&v) else {
            return Ok(v);
        };

        #[cfg(unix)]
        let path = {
            use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
            Path::new(OsStr::from_bytes(var))
        };
        // Technically `wasi` also provides an `OsStrExt` which allows conversion from bytes, but
        // since that seems to also be conditional on `target_env` for the sake of simplicity it's
        // omitted here and should be added on demand.
        #[cfg(not(unix))]
        let path = {
            match std::str::from_utf8(var) {
                Ok(s) => Path::new(s),
                Err(_) => return Ok(v), // TODO: error here
            }
        };

        let full_path = self.resolve_path(path);
        let value =
            std::fs::read(&full_path).map_err(|error| self.io_error(&full_path, path, error))?;

        Ok(Cow::Owned(value))
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
        let Some(var) = self.variable.parse_str(&v) else {
            // There is no variable in the string, the expanded variant is just the original.
            return Ok(Any::Str(v));
        };

        let path = self.resolve_path(var.as_ref());
        let value =
            std::fs::read(&path).map_err(|error| self.io_error(&path, var.as_ref(), error))?;

        let value = String::from_utf8(value)
            .map(Cow::Owned)
            .map(utils::parse)
            .unwrap_or_else(|err| Any::Bytes(Cow::Owned(err.into_bytes())));
        Ok(value)
    }
}

impl Default for FileSource {
    fn default() -> Self {
        Self::new()
    }
}
