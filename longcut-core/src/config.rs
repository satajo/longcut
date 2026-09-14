use crate::model::command::{Command, CommandError, CommandParameter};
use crate::model::effect::{EffectTemplate, ShellCommandTemplate};
use crate::model::key::{Key, KeyParseError, Modifier, Symbol};
use crate::model::layer::{Action, Layer};
use crate::model::parameter::{
    CharacterParameter, ChooseParameter, ParameterDefinitionVariant, TextParameter,
};
use itertools::Itertools;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(try_from = "ConfigSchema")]
pub struct Config {
    pub keys_back: Vec<Key>,
    pub keys_exit: Vec<Key>,
    pub root_layer: Layer,
    pub app_specific_layers: Vec<ApplicationConfig>,
}

#[derive(Debug)]
pub struct ApplicationConfig {
    pub pattern: regex::Regex,
    pub root_layer: Layer,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigSchema {
    keys_back: Option<OneOrManySchema<KeySchema>>,
    keys_exit: Option<OneOrManySchema<KeySchema>>,
    layers: Option<Vec<LayerSchema>>,
    commands: Option<Vec<CommandSchema>>,
    #[serde(default)]
    app_specific_layers: Vec<ApplicationConfigSchema>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplicationConfigSchema {
    #[serde(rename = "match")]
    pattern: String,
    layers: Option<Vec<LayerSchema>>,
    commands: Option<Vec<CommandSchema>>,
}

impl TryFrom<ConfigSchema> for Config {
    type Error = String;

    fn try_from(value: ConfigSchema) -> Result<Self, Self::Error> {
        let keys_back: Vec<Key> = match value.keys_back {
            None => vec![],
            Some(keys) => keys.try_into()?,
        };

        let keys_exit: Vec<Key> = match value.keys_exit {
            None => vec![Key::new(Symbol::ESCAPE)],
            Some(keys) => keys.try_into()?,
        };

        let root_layer = try_parse_layer("Root".to_string(), value.layers, value.commands)?;
        reject_shadowed_shortcuts(&root_layer, &keys_exit, &keys_back)?;

        let mut app_specific_layers = Vec::new();
        for app_schema in value.app_specific_layers {
            let pattern = regex::Regex::new(&app_schema.pattern)
                .map_err(|e| format!("Invalid regex pattern {:?}: {e}", app_schema.pattern))?;
            let root_layer = try_parse_layer(
                app_schema.pattern.clone(),
                app_schema.layers,
                app_schema.commands,
            )?;
            reject_shadowed_shortcuts(&root_layer, &keys_exit, &keys_back)?;
            app_specific_layers.push(ApplicationConfig {
                pattern,
                root_layer,
            });
        }

        Ok(Self {
            keys_back,
            keys_exit,
            root_layer,
            app_specific_layers,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
struct CommandSchema {
    pub name: String,
    pub shortcut: KeySchema,
    pub steps: Vec<StepSchema>,
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

        let mut steps: Vec<EffectTemplate> = value
            .steps
            .into_iter()
            .map(|s| ShellCommandTemplate::try_from(s).map(EffectTemplate::ShellCommand))
            .collect::<Result<Vec<_>, _>>()?;
        if value.is_synchronous {
            for step in &mut steps {
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
#[serde(deny_unknown_fields)]
struct StepSchema {
    bash: String,
}

impl TryFrom<StepSchema> for ShellCommandTemplate {
    type Error = String;

    fn try_from(value: StepSchema) -> Result<Self, Self::Error> {
        ShellCommandTemplate::new(&value.bash)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub options: Option<Vec<String>>,
    pub generate_options: Option<GenerateOptionsSchema>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GenerateOptionsSchema {
    pub command: String,
    pub split_by: Option<String>,
}

impl TryFrom<ParameterSchema> for CommandParameter {
    type Error = String;

    fn try_from(value: ParameterSchema) -> Result<Self, Self::Error> {
        let parameter_type = match value.type_.as_str() {
            "character" => ParameterDefinitionVariant::Character(CharacterParameter),
            "text" => ParameterDefinitionVariant::Text(TextParameter),
            "choose" => {
                let mut gen_options_command: Option<String> = None;
                let mut gen_options_split_by: Option<String> = None;

                if let Some(dynamic_config) = value.generate_options {
                    gen_options_command = Some(dynamic_config.command);
                    gen_options_split_by = dynamic_config.split_by;
                }

                match ChooseParameter::new(value.options, gen_options_command, gen_options_split_by)
                {
                    Ok(parameter) => ParameterDefinitionVariant::Choose(parameter),
                    Err(error) => {
                        return Err(format!("Invalid 'choose' parameter configuration: {error}"));
                    }
                }
            }

            otherwise => Err(format!("parameter type {otherwise} is unsupported"))?,
        };

        Ok(CommandParameter::new(value.name, parameter_type))
    }
}

/// A key as the configuration spells it: a symbol name alone, or a symbol name with modifiers.
/// Adapters with keys of their own deserialize them through this schema, so that every key is
/// spelled and reported the same way.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum KeySchema {
    Key(SymbolSchema),
    KeyAndModifiers(KeyAndModifiersSchema),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyAndModifiersSchema {
    key: SymbolSchema,
    modifiers: OneOrManySchema<ModifierSchema>,
}

impl TryFrom<KeySchema> for Key {
    type Error = String;

    fn try_from(value: KeySchema) -> Result<Self, Self::Error> {
        match value {
            KeySchema::Key(key) => key.try_into().map(Key::new),
            KeySchema::KeyAndModifiers(KeyAndModifiersSchema { key, modifiers }) => {
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
pub struct SymbolSchema(String);

impl TryFrom<SymbolSchema> for Symbol {
    type Error = String;

    fn try_from(value: SymbolSchema) -> Result<Self, Self::Error> {
        value.0.parse().map_err(|e: KeyParseError| e.to_string())
    }
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct ModifierSchema(String);

impl TryFrom<ModifierSchema> for Modifier {
    type Error = String;

    fn try_from(value: ModifierSchema) -> Result<Self, Self::Error> {
        value.0.parse().map_err(|e: KeyParseError| e.to_string())
    }
}

/// `OneOrMany` permits a value to be defined either in a list format or as a single item, with either
/// one being able to be converted into a `Vec<T>` using the `TryFrom` implementation.
///
/// The list form is tried first: serde also accepts a sequence as the fields of a struct, so a
/// two-item list of keys would otherwise deserialize as a single `{key, modifiers}` item.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OneOrManySchema<T> {
    Many(Vec<T>),
    One(T),
}

impl<T, S: TryFrom<T>> TryFrom<OneOrManySchema<T>> for Vec<S> {
    type Error = S::Error;

    fn try_from(value: OneOrManySchema<T>) -> Result<Self, Self::Error> {
        match value {
            OneOrManySchema::One(x) => vec![x.try_into()].into_iter().try_collect(),
            OneOrManySchema::Many(xs) => xs
                .into_iter()
                .map(std::convert::TryInto::try_into)
                .try_collect(),
        }
    }
}

/// Serde workaround for boolean default values
fn default_true() -> bool {
    true
}

/// Rejects shortcuts the navigation loop consumes before consulting the layer: exit keys at every
/// depth, and back keys everywhere below the root layer.
fn reject_shadowed_shortcuts(
    root_layer: &Layer,
    keys_exit: &[Key],
    keys_back: &[Key],
) -> Result<(), String> {
    fn find_shadowed<'a>(
        layer: &'a Layer,
        keys_exit: &[Key],
        keys_back: &[Key],
        is_root: bool,
    ) -> Option<(&'a Key, &'a str, &'static str)> {
        for (shortcut, action) in layer.shortcuts.iter() {
            if keys_exit.contains(shortcut) {
                return Some((shortcut, &layer.name, "keys_exit"));
            }
            if !is_root && keys_back.contains(shortcut) {
                return Some((shortcut, &layer.name, "keys_back"));
            }
            if let Action::Branch(sublayer) = action
                && let Some(found) = find_shadowed(sublayer, keys_exit, keys_back, false)
            {
                return Some(found);
            }
        }
        None
    }

    match find_shadowed(root_layer, keys_exit, keys_back, true) {
        Some((shortcut, layer_name, shadowing_setting)) => Err(format!(
            "Shortcut {shortcut} in layer {layer_name:?} can never be pressed because it is \
             consumed by {shadowing_setting}"
        )),
        None => Ok(()),
    }
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
                let error_message = format!(
                    "Can not assign layer to key {conflicting_key} because of an existing binding!"
                );
                return Err(error_message);
            }
        }
    }

    if let Some(schemas) = commands {
        for schema in schemas {
            let (shortcut, command): (Key, Command) = schema.try_into()?;
            if let Err((conflicting_key, _)) = layer.add_command(shortcut, command) {
                let error_message = format!(
                    "Could not assign command to key {conflicting_key} because of an existing binding!"
                );
                return Err(error_message);
            }
        }
    }

    Ok(layer)
}
