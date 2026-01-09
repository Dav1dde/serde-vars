use std::{borrow::Cow, fmt};

use crate::source::Any;

#[derive(Debug)]
pub struct Variable {
    pub prefix: String,
    pub suffix: String,
}

impl Variable {
    pub fn parse_str<'a>(&self, v: &'a str) -> Option<&'a str> {
        v.strip_prefix(&self.prefix)?.strip_suffix(&self.suffix)
    }

    pub fn parse_bytes<'a>(&self, v: &'a [u8]) -> Option<&'a [u8]> {
        v.strip_prefix(self.prefix.as_bytes())?
            .strip_suffix(self.suffix.as_bytes())
    }

    pub fn fmt<'a, T>(&'a self, v: T) -> impl fmt::Display + use<'a, T>
    where
        T: fmt::Display,
    {
        struct D<'a, T> {
            this: &'a Variable,
            v: T,
        }

        impl<T> fmt::Display for D<'_, T>
        where
            T: fmt::Display,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{}{}", self.this.prefix, self.v, self.this.suffix)
            }
        }

        D { this: self, v }
    }
}

impl Default for Variable {
    fn default() -> Self {
        Self {
            prefix: "${".to_owned(),
            suffix: "}".to_owned(),
        }
    }
}

pub fn parse(s: Cow<'_, str>) -> Any<'_> {
    match s.as_ref() {
        "true" => Any::Bool(true),
        "false" => Any::Bool(false),
        // Try in order:
        //  - parse f64
        //  - parse u64
        //  - use the literal string
        v => v
            .parse()
            .map(Any::U64)
            .or_else(|_| v.parse().map(Any::I64))
            .or_else(|_| v.parse().map(Any::F64))
            .unwrap_or(Any::Str(s)),
    }
}
