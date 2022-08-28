mod yaml;

use crate::config::yaml::{OneOrMany, Shortcut, YamlConfiguration};
use crate::config::ConfigurationError::Semantic;
use itertools::Itertools;
use longcut_core::model::command::{
    Command, CommandError, InstructionTemplate, ParameterDeclaration,
};
use longcut_core::model::key::{Key, Modifier, Symbol};
use longcut_core::model::layer::Layer;
use longcut_core::Configuration;
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

fn conflicting_key_bindings(layer: &Layer, key: &Key) -> ConfigurationError {
    let message = format!(
        "Multiple actions registered under the same key {:?} in layer {}",
        key, layer.name
    );
    Semantic(message)
}

fn parse_root_layer(data: &yaml::RootLayer) -> Result<Layer, ConfigurationError> {
    let name = data
        .name
        .as_ref()
        .cloned()
        .unwrap_or_else(|| "Root".to_string());

    let mut layer = Layer::new(name);

    if let Some(sublayers) = data.layers.as_ref() {
        layer = register_layer_sublayers(layer, sublayers)?;
    }

    if let Some(commands) = data.commands.as_ref() {
        layer = register_layer_commands(layer, commands)?;
    }

    Ok(layer)
}

fn register_layer_sublayers(
    mut layer: Layer,
    data: &[yaml::Layer],
) -> Result<Layer, ConfigurationError> {
    for sublayer_data in data {
        let shortcut = parse_shortcut(&sublayer_data.shortcut)?;
        let sublayer = parse_layer(sublayer_data)?;
        if let Err((key, _)) = layer.add_layer(shortcut, sublayer) {
            return Err(conflicting_key_bindings(&layer, &key));
        }
    }
    Ok(layer)
}

fn register_layer_commands(
    mut layer: Layer,
    data: &[yaml::Command],
) -> Result<Layer, ConfigurationError> {
    for command_data in data {
        let shortcut = parse_shortcut(&command_data.shortcut)?;
        let command = parse_command(command_data)?;
        if let Err((key, _)) = layer.add_command(shortcut, command) {
            return Err(conflicting_key_bindings(&layer, &key));
        }
    }
    Ok(layer)
}

fn parse_layer(data: &yaml::Layer) -> Result<Layer, ConfigurationError> {
    let mut layer = Layer::new(data.name.clone());

    if let Some(sublayers) = data.layers.as_ref() {
        layer = register_layer_sublayers(layer, sublayers)?;
    }

    if let Some(commands) = data.commands.as_ref() {
        layer = register_layer_commands(layer, commands)?;
    }

    Ok(layer)
}

fn parse_command(data: &yaml::Command) -> Result<Command, ConfigurationError> {
    // Step parsing
    let parse_step = |step_data: &yaml::Step| -> Result<InstructionTemplate, ConfigurationError> {
        let mut step =
            InstructionTemplate::new(step_data.clone()).map_err(ConfigurationError::Semantic)?;
        step.set_synchronous(data.is_synchronous);
        Ok(step)
    };

    let steps = match &data.steps {
        OneOrMany::One(step) => vec![parse_step(step)?],
        OneOrMany::Many(steps) => steps.iter().map(parse_step).collect::<Result<_, _>>()?,
    };

    // Parameter parsing
    let parse_parameter =
        |parameter_data: &yaml::Parameter| -> Result<ParameterDeclaration, ConfigurationError> {
            let parameter_name = parameter_data.name.clone();
            match parameter_data.type_.as_ref() {
                "character" => Ok(ParameterDeclaration::character(parameter_name)),
                "text" => Ok(ParameterDeclaration::text(parameter_name)),
                otherwise => {
                    let message = format!("parameter type {} is unsupported", otherwise);
                    Err(ConfigurationError::Semantic(message))
                }
            }
        };

    let parameters = match &data.parameters {
        None => vec![],
        Some(one_or_many) => match one_or_many {
            OneOrMany::One(parameter) => vec![parse_parameter(parameter)?],
            OneOrMany::Many(parameters) => parameters
                .iter()
                .map(parse_parameter)
                .collect::<Result<_, _>>()?,
        },
    };

    // Command building
    let mut command =
        Command::new(data.name.clone(), steps, parameters).map_err(|err| match err {
            CommandError::NoStepsProvided => {
                ConfigurationError::Semantic("command has no associated steps".into())
            }
            CommandError::MissingParameter(idx) => {
                let message = format!("required {}. parameter was not declared", idx);
                ConfigurationError::Semantic(message)
            }
            CommandError::UnusedParameter(idx) => {
                let message = format!("declared {}. parameter is unused", idx);
                ConfigurationError::Semantic(message)
            }
        })?;
    command.set_final(data.is_final);
    Ok(command)
}

fn parse_shortcuts(data: &OneOrMany<yaml::Shortcut>) -> Result<Vec<Key>, ConfigurationError> {
    match data {
        OneOrMany::One(shortcut) => parse_shortcut(shortcut).map(|key| vec![key]),
        OneOrMany::Many(shortcuts) => shortcuts.iter().map(parse_shortcut).try_collect(),
    }
}

fn parse_shortcut(data: &yaml::Shortcut) -> Result<Key, ConfigurationError> {
    fn parse_key_symbol(symbol: &str) -> Result<Key, ConfigurationError> {
        Symbol::try_from(symbol).map(Key::new).map_err(|_| {
            ConfigurationError::Semantic(format!("{} is not a valid key symbol", symbol))
        })
    }

    fn parse_key_modifier(modifier: &str) -> Result<Modifier, ConfigurationError> {
        modifier.try_into().map_err(|_| {
            ConfigurationError::Semantic(format!("{} is not a valid modifier key", modifier))
        })
    }

    match data {
        Shortcut::Key(key) => parse_key_symbol(key),
        Shortcut::KeyAndModifiers { key, modifiers } => {
            let mut key = parse_key_symbol(key)?;
            match modifiers {
                OneOrMany::One(modifier) => {
                    let modifier = parse_key_modifier(modifier)?;
                    key.add_modifier(modifier);
                }
                OneOrMany::Many(modifiers) => {
                    for modifier in modifiers.iter() {
                        let modifier = parse_key_modifier(modifier)?;
                        key.add_modifier(modifier);
                    }
                }
            }

            Ok(key)
        }
    }
}