//! YAML configuration loading, with the section of each module located and deserialized through the `Module` trait.

mod module;

pub use module::{ConfigError, ConfigModule, InitError, Module};
