use longcut_config::{ConfigError, ConfigModule, InitError};
use serde::Deserialize;
use std::path::PathBuf;

#[test]
fn can_initialize_config_module_with_valid_yaml_file() {
    let file_path = path_to_test_data_file("valid_config.yaml");
    let result = ConfigModule::new(file_path);
    assert!(result.is_ok());
}

#[test]
fn initializing_config_module_with_missing_file_is_an_error() {
    let file_path = path_to_test_data_file("this_file_does_not_exist.yaml");
    let result = ConfigModule::new(file_path);
    assert!(result.is_err());

    let error = result.err().unwrap();
    assert!(matches!(error, InitError::FileNotFound));
}

#[test]
fn initializing_config_module_with_invalid_config_file_is_an_error() {
    let file_path = path_to_test_data_file("invalid_config.yaml");
    let result = ConfigModule::new(file_path);
    assert!(result.is_err());

    let error = result.err().unwrap();
    assert!(matches!(error, InitError::ParsingError(..)));
}

#[test]
fn can_get_configuration_for_defined_top_level_key() {
    let file_path = path_to_test_data_file("valid_config.yaml");
    let module = ConfigModule::new(file_path).unwrap();

    #[derive(Deserialize)]
    struct SimpleExample {
        value: String,
    }

    let result = module.config_for_key::<SimpleExample>("simple");
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.value, "Simple example");
}

#[test]
fn missing_top_level_key_results_in_an_error() {
    let file_path = path_to_test_data_file("valid_config.yaml");
    let module = ConfigModule::new(file_path).unwrap();

    #[derive(Deserialize)]
    struct SimpleExample {
        #[allow(dead_code)]
        value: String,
    }

    let result = module.config_for_key::<SimpleExample>("key_not_found");
    assert!(result.is_err());

    let error = result.err().unwrap();
    assert!(matches!(error, ConfigError::KeyNotFound))
}

#[test]
fn deserialization_error_is_forwarded_correctly() {
    let file_path = path_to_test_data_file("valid_config.yaml");
    let module = ConfigModule::new(file_path).unwrap();

    #[derive(Deserialize)]
    struct InvalidExample {
        #[allow(dead_code)]
        value: u32,
    }

    let result = module.config_for_key::<InvalidExample>("simple");
    assert!(result.is_err());

    let error = result.err().unwrap();
    assert!(matches!(error, ConfigError::DeserializationError(..)))
}

/// Get a path to the specified file under the tests directory.
fn path_to_test_data_file(file_name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/");
    path.push(file_name);
    path
}
