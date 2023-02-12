use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
#[serde(try_from = "ConfigSchema")]
pub struct Config {
    pub default_timeout: Duration,
}

#[derive(Debug, Deserialize)]
struct ConfigSchema {
    default_timeout_ms: u64,
}

impl TryFrom<ConfigSchema> for Config {
    type Error = String;

    fn try_from(value: ConfigSchema) -> Result<Self, Self::Error> {
        let default_timeout = Duration::from_millis(value.default_timeout_ms);
        Ok(Config { default_timeout })
    }
}
