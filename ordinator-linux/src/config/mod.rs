mod yaml;

use crate::config::yaml::{OneOrMany, Shortcut, YamlConfiguration};
use itertools::Itertools;
use ordinator_core::model::command::{Command, Step};
use ordinator_core::model::key::KeyPress;
use ordinator_core::model::layer::Layer;
use ordinator_core::Configuration;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub enum ConfigurationError {
    File(String),
    Syntax(String),
    Semantic(String),
}

pub fn read_config(path: &Path) -> Result<Configuration, ConfigurationError> {
    let file = read_config_file(path)?;
    let yaml_config = parse_yaml_config_file(&file)?;
    println!("{:?}", &yaml_config);
    parse_configuration(&yaml_config)
}

fn read_config_file(path: &Path) -> Result<File, ConfigurationError> {
    File::open(&path).map_err(|e| ConfigurationError::File(e.to_string()))
}

fn parse_yaml_config_file(file: &File) -> Result<YamlConfiguration, ConfigurationError> {
    YamlConfiguration::parse(file).map_err(|e| ConfigurationError::Syntax(e.to_string()))
}

fn parse_configuration(yaml: &YamlConfiguration) -> Result<Configuration, ConfigurationError> {
    let keys_activate = parse_shortcuts(&yaml.keys.activate)?;

    // Back keys are optional and default to none.
    let keys_back = if let Some(keys) = &yaml.keys.back {
        parse_shortcuts(keys)?
    } else {
        vec![]
    };

    // Deactivate keys are optional and default to activation keys.s
    let keys_deactivate = if let Some(keys) = &yaml.keys.deactivate {
        parse_shortcuts(keys)?
    } else {
        keys_activate.clone()
    };

    let config = Configuration {
        keys_activate,
        keys_back,
        keys_deactivate,
        root_layer: parse_root_layer(&yaml.root)?,
    };
    Ok(config)
}

fn parse_root_layer(data: &yaml::RootLayer) -> Result<Layer, ConfigurationError> {
    let name = data
        .name
        .as_ref()
        .cloned()
        .unwrap_or_else(|| "Root".to_string());

    let mut layer = Layer::new(name);

    for sublayer_data in data.layers.as_ref().unwrap_or(&vec![]) {
        let shortcut = parse_shortcut(&sublayer_data.shortcut)?;
        let sublayer = parse_layer(sublayer_data)?;
        layer.add_layer(shortcut, sublayer);
    }

    for command_data in data.commands.as_ref().unwrap_or(&vec![]) {
        let shortcut = parse_shortcut(&command_data.shortcut)?;
        let command = parse_command(command_data)?;
        layer.add_command(shortcut, command)
    }

    Ok(layer)
}

fn parse_layer(data: &yaml::Layer) -> Result<Layer, ConfigurationError> {
    let mut layer = Layer::new(data.name.clone());

    for sublayer_data in data.layers.as_ref().unwrap_or(&vec![]) {
        let shortcut = parse_shortcut(&sublayer_data.shortcut)?;
        let sublayer = parse_layer(sublayer_data)?;
        layer.add_layer(shortcut, sublayer);
    }

    for command_data in data.commands.as_ref().unwrap_or(&vec![]) {
        let shortcut = parse_shortcut(&command_data.shortcut)?;
        let command = parse_command(command_data)?;
        layer.add_command(shortcut, command)
    }

    Ok(layer)
}

fn parse_command(data: &yaml::Command) -> Result<Command, ConfigurationError> {
    let steps = match &data.steps {
        OneOrMany::One(step) => vec![Step::new(step.clone())],
        OneOrMany::Many(steps) => steps.iter().cloned().map(Step::new).collect(),
    };
    Ok(Command::new(data.name.clone(), steps))
}

fn parse_shortcuts(data: &OneOrMany<yaml::Shortcut>) -> Result<Vec<KeyPress>, ConfigurationError> {
    match data {
        OneOrMany::One(shortcut) => parse_shortcut(shortcut).map(|key| vec![key]),
        OneOrMany::Many(shortcuts) => shortcuts.iter().map(parse_shortcut).try_collect(),
    }
}

fn parse_shortcut(data: &yaml::Shortcut) -> Result<KeyPress, ConfigurationError> {
    fn parse_key_str(symbol: &str) -> Result<KeyPress, ConfigurationError> {
        symbol.try_into().map_err(ConfigurationError::Semantic)
    }

    match data {
        Shortcut::Key(symbol) => parse_key_str(symbol),
        Shortcut::KeyAndModifiers { key, modifiers } => {
            let key = parse_key_str(key)?;
            Ok(key)
        }
    }
}
