use crate::model::command::{Command, CommandError, CommandParameter, InstructionTemplate};
use crate::model::key::{Key, Modifier, Symbol};
use crate::model::layer::Layer;
use crate::model::parameter::Parameter;
use itertools::Itertools;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(try_from = "ConfigSchema")]
pub struct Config {
    pub keys_activate: Vec<Key>,
    pub keys_back: Vec<Key>,
    pub keys_deactivate: Vec<Key>,
    pub root_layer: Layer,
}

#[derive(Debug, Deserialize)]
struct ConfigSchema {
    keys_activate: OneOrManySchema<KeySchema>,
    keys_back: Option<OneOrManySchema<KeySchema>>,
    keys_deactivate: Option<OneOrManySchema<KeySchema>>,
    layers: Option<Vec<LayerSchema>>,
    commands: Option<Vec<CommandSchema>>,
}

impl TryFrom<ConfigSchema> for Config {
    type Error = String;

    fn try_from(value: ConfigSchema) -> Result<Self, Self::Error> {
        let keys_activate: Vec<Key> = value.keys_activate.try_into()?;

        let keys_back: Vec<Key> = match value.keys_back {
            None => vec![],
            Some(keys) => keys.try_into()?,
        };

        let keys_deactivate: Vec<Key> = match value.keys_deactivate {
            None => keys_activate.clone(),
            Some(keys) => keys.try_into()?,
        };

        let root_layer = try_parse_layer("Root".to_string(), value.layers, value.commands)?;

        Ok(Self {
            keys_activate,
            keys_back,
            keys_deactivate,
            root_layer,
        })
    }
}

#[derive(Debug, Deserialize)]
struct LayerSchema {
    layers: Option<Vec<LayerSchema>>,
    commands: Option<Vec<CommandSchema>>,
    shortcut: KeySchema,
    name: String,
}

impl TryFrom<LayerSchema> for (Key, Layer) {
    type Error = String;

    fn try_from(value: LayerSchema) -> Result<Self, Self::Error> {
        let shortcut: Key = value.shortcut.try_into()?;
        let layer = try_parse_layer(value.name, value.layers, value.commands)?;
        Ok((shortcut, layer))
    }
}

#[derive(Debug, Deserialize)]
struct CommandSchema {
    pub name: String,
    pub shortcut: KeySchema,
    pub steps: OneOrManySchema<StepSchema>,
    pub parameters: Option<OneOrManySchema<ParameterSchema>>,
    #[serde(rename = "final")]
    #[serde(default = "default_true")]
    pub is_final: bool,

    #[serde(rename = "synchronous")]
    #[serde(default = "default_true")]
    pub is_synchronous: bool,
}

impl TryFrom<CommandSchema> for (Key, Command) {
    type Error = String;

    fn try_from(value: CommandSchema) -> Result<Self, Self::Error> {
        let shortcut: Key = value.shortcut.try_into()?;

        let mut steps: Vec<InstructionTemplate> = value.steps.try_into()?;
        if value.is_synchronous {
            for step in steps.iter_mut() {
                step.set_synchronous(true);
            }
        }

        let parameters: Vec<CommandParameter> = match value.parameters {
            None => vec![],
            Some(xs) => xs.try_into()?,
        };

        let mut command = Command::new(value.name, steps, parameters).map_err(|err| match err {
            CommandError::NoStepsProvided => "Command has no associated steps".to_string(),
            CommandError::MissingParameter(idx) => {
                format!("required {idx}. parameter was not declared")
            }
            CommandError::UnusedParameter(idx) => {
                format!("declared {idx}. parameter is unused")
            }
        })?;

        command.set_final(value.is_final);
        Ok((shortcut, command))
    }
}

#[derive(Debug, Deserialize)]
struct StepSchema(String);

impl TryFrom<StepSchema> for InstructionTemplate {
    type Error = String;

    fn try_from(value: StepSchema) -> Result<Self, Self::Error> {
        InstructionTemplate::new(value.0)
    }
}

#[derive(Debug, Deserialize)]
struct ParameterSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub options: Option<Vec<String>>,
}

impl TryFrom<ParameterSchema> for CommandParameter {
    type Error = String;

    fn try_from(value: ParameterSchema) -> Result<Self, Self::Error> {
        let parameter_type = match value.type_.as_str() {
            "character" => Parameter::Character,
            "text" => Parameter::Text,
            "choose" => {
                if let Some(options) = value.options {
                    Parameter::Choose(options)
                } else {
                    return Err("Parameter options not provided!".to_string());
                }
            }
            otherwise => Err(format!("parameter type {otherwise} is unsupported"))?,
        };

        Ok(CommandParameter::new(value.name, parameter_type))
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum KeySchema {
    Key(SymbolSchema),
    KeyAndModifiers {
        key: SymbolSchema,
        modifiers: OneOrManySchema<ModifierSchema>,
    },
}

impl TryFrom<KeySchema> for Key {
    type Error = String;

    fn try_from(value: KeySchema) -> Result<Self, Self::Error> {
        match value {
            KeySchema::Key(key) => key.try_into().map(Key::new),
            KeySchema::KeyAndModifiers { key, modifiers } => {
                let mut symbol = key.try_into().map(Key::new)?;

                for modifier in TryInto::<Vec<Modifier>>::try_into(modifiers)? {
                    symbol.add_modifier(modifier);
                }

                Ok(symbol)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct SymbolSchema(String);

impl TryFrom<SymbolSchema> for Symbol {
    type Error = String;

    fn try_from(value: SymbolSchema) -> Result<Self, Self::Error> {
        value.0.as_str().try_into().map_err(|e: &str| e.to_string())
    }
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct ModifierSchema(String);

impl TryFrom<ModifierSchema> for Modifier {
    type Error = String;

    fn try_from(value: ModifierSchema) -> Result<Self, Self::Error> {
        value.0.as_str().try_into().map_err(|e: &str| e.to_string())
    }
}

/// OneOrMany permits a value to be defined either in a list format or as a single item, with either
/// one being able to be converted into a Vec<T> using the TryFrom implementation.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OneOrManySchema<T> {
    One(T),
    Many(Vec<T>),
}

impl<T, S: TryFrom<T>> TryFrom<OneOrManySchema<T>> for Vec<S> {
    type Error = S::Error;

    fn try_from(value: OneOrManySchema<T>) -> Result<Self, Self::Error> {
        match value {
            OneOrManySchema::One(x) => vec![x.try_into()].into_iter().try_collect(),
            OneOrManySchema::Many(xs) => xs.into_iter().map(|x| x.try_into()).try_collect(),
        }
    }
}

/// Serde workaround for boolean default values
fn default_true() -> bool {
    true
}

/// Parses a Layer out of the provided data.
fn try_parse_layer(
    name: String,
    layers: Option<Vec<LayerSchema>>,
    commands: Option<Vec<CommandSchema>>,
) -> Result<Layer, String> {
    let mut layer = Layer::new(name);

    if let Some(schemas) = layers {
        for schema in schemas {
            let (shortcut, sublayer): (Key, Layer) = schema.try_into()?;
            if let Err((conflicting_key, _)) = layer.add_layer(shortcut, sublayer) {
                let error_message = format!("Can not assign layer to key {conflicting_key:?} because of an existing binding!");
                return Err(error_message);
            }
        }
    }

    if let Some(schemas) = commands {
        for schema in schemas {
            let (shortcut, command): (Key, Command) = schema.try_into()?;
            if let Err((conflicting_key, _)) = layer.add_command(shortcut, command) {
                let error_message = format!("Could not assign command to key {conflicting_key:?} because of an existing binding!");
                return Err(error_message);
            }
        }
    }

    Ok(layer)
}
