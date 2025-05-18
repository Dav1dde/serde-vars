Serde Vars
==========

[![Crates.io][crates-badge]][crates-url]
[![License][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![docs.rs][docsrs-badge]][docsrs-url]

[crates-badge]: https://img.shields.io/crates/v/serde-vars.svg
[crates-url]: https://crates.io/crates/serde-vars
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/Dav1dde/serde-vars/blob/master/LICENSE
[actions-badge]: https://github.com/Dav1dde/serde-vars/workflows/CI/badge.svg
[actions-url]: https://github.com/Dav1dde/serde-vars/actions?query=workflow%3ACI+branch%3Amaster
[docsrs-badge]: https://img.shields.io/docsrs/serde-vars
[docsrs-url]: https://docs.rs/serde-vars


Conveniently expose (environment) variables to your [serde](https://docs.rs/serde/) based data structures,
like configurations.


```json
{
    "redis": {
        "host": "127.0.0.1",
        "port": 6379,
        "username": "${REDIS_USERNAME}",
        "password": "${REDIS_PASSWORD}"
    }
}
```

The configuration file contains the variables, no need to decide on variable mappings at compile time.


```rs
#[derive(Debug, serde::Deserialize)]
struct Config {
    redis: Redis,
}

#[derive(Debug, serde::Deserialize)]
struct Redis {
    host: String,
    port: u16,
    username: String,
    password: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.json".to_owned());

    let config = std::fs::read_to_string(&config_path)?;
    
    let mut source = serde_vars::EnvSource::default();
    let mut de = serde_json::Deserializer::from_str(&config);
    let config: Config = serde_vars::deserialize(&mut de, &mut source)?;

    println!("{config:#?}");

    Ok(())
}
```
