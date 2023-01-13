use crate::module::ConfigError::{DeserializationError, KeyNotFound};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub trait Module {
    /// Human-readable string which uniquely identifies this module.
    ///
    /// This identifier is implicitly used as a key or identifier in various contexts. Examples
    /// include module configuration and logging.
    const IDENTIFIER: &'static str;

    type Config: DeserializeOwned;
}

type TopLevelConfig = HashMap<String, serde_yaml::Value>;

/// Provides methods access to the contents of the wrapped configuration file.
pub struct ConfigModule {
    raw_config: TopLevelConfig,
}

#[derive(Debug)]
pub enum InitError {
    /// The configuration file did not exist.
    FileNotFound,

    /// The configuration file was deserializable to the [TopLevelConfig] schema.
    ParsingError(String),
}

#[derive(Debug)]
pub enum ConfigError {
    /// The requested top-level key was not found in the configuration.
    KeyNotFound,

    /// The configuration was not deserializable to the specified schema.
    DeserializationError(String),
}

impl ConfigModule {
    pub fn new(config_file: impl AsRef<Path>) -> Result<Self, InitError> {
        let file_contents =
            read_file_to_string(config_file.as_ref()).map_err(|_| InitError::FileNotFound)?;
        let raw_config = serde_yaml::from_str(&file_contents)
            .map_err(|e| InitError::ParsingError(e.to_string()))?;
        Ok(Self { raw_config })
    }

    /// Parses the configuration from under the specified key into the provided schema.
    pub fn config_for_key<T: DeserializeOwned>(&self, key: &str) -> Result<T, ConfigError> {
        let raw = match self.raw_config.get(key) {
            Some(value) => value,
            None => return Err(KeyNotFound),
        };

        serde_yaml::from_value(raw.clone()).map_err(|e| DeserializationError(e.to_string()))
    }

    /// Uses the [Module] metadata to deserialize and parse its configuration.
    pub fn config_for_module<M: Module>(&self) -> Result<M::Config, ConfigError> {
        self.config_for_key::<M::Config>(M::IDENTIFIER)
    }
}

fn read_file_to_string(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}
