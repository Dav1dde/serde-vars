//! Conveniently expose (environment) variables to your [`serde`] based data structures,
//! like configurations.
//!
//! The main goal of this library is to allow for a very simple but yet powerful mechanism
//! of dynamically loading single configuration values from (environment) variables.
//! It is ideal, when you need to include sensitive values in configurations, but don't want
//! to or can't (e.g. due to repeated or dynamic values) setup manual environment mappings at compile time.
//!
//! By implementing a [`serde::de::Deserializer`], this crate works independently of the
//! serialization format used and with only a few lines of modifications for any configuration.
//!
//! # Example
//!
//! User provided configuration file:
//!
//! ```json
//! {
//!     "redis": {
//!         "host": "${REDIS_HOST}",
//!         "port": "${REDIS_PORT}"
//!     }
//! }
//! ```
//!
//! The configuration file contains the variables, no need to decide on variable mappings at compile time.
//!
//!
//! ```
//! #[derive(Debug, serde::Deserialize)]
//! struct Config {
//!     redis: Redis,
//! }
//!
//! #[derive(Debug, serde::Deserialize)]
//! struct Redis {
//!     host: std::net::Ipv4Addr,
//!     port: u16,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config_path = std::env::args()
//!         .nth(1)
//!         .unwrap_or_else(|| "config.json".to_owned());
//!
//!     # let config_path = file!();
//!     let config = std::fs::read_to_string(&config_path)?;
//!     # let config = r#"{"redis": {"host": "${REDIS_HOST}", "port": "${REDIS_PORT}"}}"#;
//!     # unsafe { std::env::set_var("REDIS_HOST", "127.0.0.1"); }
//!     # unsafe { std::env::set_var("REDIS_PORT", "6379"); }
//!
//!     let mut source = serde_vars::EnvSource::default();
//!
//!     // Before: `let config: Config = serde_json::from_str(&config)?`.
//!     // Now:
//!     let mut de = serde_json::Deserializer::from_str(&config);
//!     let config: Config = serde_vars::deserialize(&mut de, &mut source)?;
//!
//!     println!("{config:#?}");
//!     # assert_eq!(config.redis.host, std::net::Ipv4Addr::new(127, 0, 0, 1));
//!     # assert_eq!(config.redis.port, 6379);
//!
//!     Ok(())
//! }
//! ```
//!
//! # String based Lookups
//!
//! `serde-vars` comes packaged with builtin variable sources for environment variables
//! ([`EnvSource`]), hash maps ([`MapSource`]) as well as a generic string based source
//! ([`StringSource`]).
//!
//! In order to guarantee a consistent parsing behaviour, the [`StringSource`] and all of its
//! dependent implementations (like [`EnvSource`]) enforce the following format for all values:
//!
//! - `true` and `false` are always interpreted as a `bool`.
//! - Any positive integer is parsed as a `u64`.
//! - Any negative integer is parsed as a `i64`.
//! - Any floating point value is parsed as a `f64`.
//! - Everything else is parsed as a string. In order to be able to specify numbers as strings,
//!   the source recognizes arbitrary values wrapped in `"` as a string. For example `"123"` is
//!   parsed as the literal string `123`.
//!
//! For more details read the [`StringSource`] documentation.
//!
//! # Alternatives
//!
//! Variable expansion is limited to primitive types and not supported for nested data structures,
//! this is currently by design. The intention of this library is not to provide another generic
//! abstraction layer for configurations. If you are looking for a much more powerful mechanism
//! to load and layer [`serde`] based configurations, you should take a look at
//! [`figment`](https://docs.rs/figment/) instead.

mod content;
mod de;
pub mod source;
mod value;

pub use self::de::Deserializer;
pub use self::source::{EnvSource, MapSource, StringSource};

/// Entry point. See [crate documentation](crate) for an example.
pub fn deserialize<'de, D, S, T>(deserializer: D, source: &mut S) -> Result<T, D::Error>
where
    D: serde::de::Deserializer<'de>,
    T: serde::de::Deserialize<'de>,
    S: source::Source,
{
    T::deserialize(self::de::Deserializer::new(deserializer, source))
}
