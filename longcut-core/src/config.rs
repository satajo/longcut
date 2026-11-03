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
            OneOrManySchema::Many(xs) => xs.into_iter().map(TryInto::try_into).try_collect(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::effect::Effect;

    fn parse(yaml: &str) -> Result<Config, String> {
        serde_norway::from_str(yaml).map_err(|error| error.to_string())
    }

    fn parse_valid(yaml: &str) -> Config {
        parse(yaml).unwrap_or_else(|error| panic!("the configuration is rejected: {error}"))
    }

    fn rejection(yaml: &str) -> String {
        match parse(yaml) {
            Ok(config) => panic!("the configuration is accepted: {config:?}"),
            Err(error) => error,
        }
    }

    fn key(name: &str) -> Key {
        Key::new(name.parse().unwrap())
    }

    fn key_with(name: &str, modifiers: &[Modifier]) -> Key {
        let mut key = key(name);
        for modifier in modifiers {
            key.add_modifier(*modifier);
        }
        key
    }

    fn command<'a>(layer: &'a Layer, shortcut: &str) -> &'a Command {
        let Some(Action::Execute(command)) = layer.resolve_shortcut(&key(shortcut)) else {
            panic!("layer {:?} has no command under {shortcut}", layer.name);
        };
        command
    }

    fn sublayer<'a>(layer: &'a Layer, shortcut: &str) -> &'a Layer {
        let Some(Action::Branch(sublayer)) = layer.resolve_shortcut(&key(shortcut)) else {
            panic!("layer {:?} has no layer under {shortcut}", layer.name);
        };
        sublayer
    }

    fn shortcuts(layer: &Layer) -> Vec<Key> {
        layer.shortcuts.keys().cloned().collect()
    }

    fn programs(command: &Command) -> Vec<(String, bool)> {
        command
            .render_effects(vec![])
            .unwrap()
            .into_iter()
            .map(
                |Effect::ShellCommand {
                     program,
                     is_synchronous,
                 }| (program, is_synchronous),
            )
            .collect()
    }

    fn parameter_names(command: &Command) -> Vec<&str> {
        command
            .get_parameters()
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect()
    }

    fn only_choose_parameter(command: &Command) -> &ChooseParameter {
        let [
            CommandParameter {
                parameter: ParameterDefinitionVariant::Choose(choose),
                ..
            },
        ] = command.get_parameters().as_slice()
        else {
            panic!("the command does not take exactly one choose parameter");
        };
        choose
    }

    fn root_shortcut(shortcut: &str) -> String {
        format!(
            "
            commands:
              - name: Greet
                shortcut: {shortcut}
                steps:
                  - bash: echo hello
            "
        )
    }

    fn root_parameters(steps: &str, parameters: &str) -> String {
        format!(
            "
            commands:
              - name: Greet
                shortcut: g
                steps: {steps}
                parameters: {parameters}
            "
        )
    }

    const MINIMAL: &str = "
        commands:
          - name: Greet
            shortcut: g
            steps:
              - bash: echo hello
        ";

    #[test]
    fn without_key_settings_escape_exits_and_no_key_goes_back() {
        let config = parse_valid(MINIMAL);
        assert_eq!(config.keys_exit, [key("Escape")]);
        assert_eq!(config.keys_back, []);
    }

    #[test]
    fn without_application_layers_there_are_none() {
        assert!(parse_valid(MINIMAL).app_specific_layers.is_empty());
    }

    #[test]
    fn an_empty_section_has_an_empty_root_layer() {
        assert!(shortcuts(&parse_valid("{}").root_layer).is_empty());
    }

    #[test]
    fn a_root_command_is_bound_under_its_shortcut_in_the_layer_named_root() {
        let config = parse_valid(MINIMAL);
        assert_eq!(config.root_layer.name, "Root");
        assert_eq!(shortcuts(&config.root_layer), [key("g")]);
        assert_eq!(command(&config.root_layer, "g").name, "Greet");
    }

    #[test]
    fn a_command_without_parameters_takes_none() {
        let config = parse_valid(MINIMAL);
        assert!(command(&config.root_layer, "g").get_parameters().is_empty());
    }

    #[test]
    fn a_command_is_final_by_default() {
        let config = parse_valid(MINIMAL);
        assert!(command(&config.root_layer, "g").is_final);
    }

    #[test]
    fn a_command_can_be_made_non_final() {
        let config = parse_valid(
            "
            commands:
              - name: Greet
                shortcut: g
                steps:
                  - bash: echo hello
                final: false
            ",
        );
        assert!(!command(&config.root_layer, "g").is_final);
    }

    #[test]
    fn an_unknown_option_is_rejected() {
        let error = rejection("keys_quit: q");
        assert!(error.contains("unknown field `keys_quit`"), "{error}");
    }

    #[test]
    fn a_single_character_shortcut_names_the_key_that_types_it() {
        let config = parse_valid(&root_shortcut("\"!\""));
        assert_eq!(shortcuts(&config.root_layer), [key("exclam")]);
    }

    #[test]
    fn an_xkb_keysym_name_shortcut_names_that_keysym_in_any_of_its_spellings() {
        let config = parse_valid(&root_shortcut("Page_Up"));
        assert_eq!(shortcuts(&config.root_layer), [key("Prior")]);
    }

    #[test]
    fn a_shortcut_carries_its_modifiers() {
        let config = parse_valid(&root_shortcut("{ key: Up, modifiers: [Shift, Control] }"));
        assert_eq!(
            shortcuts(&config.root_layer),
            [key_with("Up", &[Modifier::Shift, Modifier::Control])]
        );
    }

    #[test]
    fn a_single_modifier_may_be_given_without_a_list() {
        let config = parse_valid(&root_shortcut("{ key: a, modifiers: Super }"));
        assert_eq!(
            shortcuts(&config.root_layer),
            [key_with("a", &[Modifier::Super])]
        );
    }

    #[test]
    fn a_shortcut_that_is_not_an_xkb_keysym_name_is_rejected() {
        let error = rejection(&root_shortcut("capslock"));
        assert!(
            error.contains("\"capslock\" is not an xkb keysym name"),
            "{error}"
        );
    }

    #[test]
    fn a_digit_shortcut_must_be_quoted_since_yaml_reads_a_bare_digit_as_a_number() {
        assert_eq!(
            shortcuts(&parse_valid(&root_shortcut("\"1\"")).root_layer),
            [key("1")]
        );
        let error = rejection(&root_shortcut("1"));
        assert!(
            error.contains("did not match any variant of untagged enum KeySchema"),
            "{error}"
        );
    }

    #[test]
    fn a_modifier_that_is_not_a_modifier_name_is_rejected() {
        let error = rejection(&root_shortcut("{ key: a, modifiers: [shift] }"));
        assert!(
            error.contains("\"shift\" is not a modifier name"),
            "{error}"
        );
    }

    #[test]
    fn back_and_exit_keys_may_be_given_as_one_key() {
        let config = parse_valid(
            "
            keys_back: BackSpace
            keys_exit: { key: c, modifiers: Control }
            ",
        );
        assert_eq!(config.keys_back, [key("BackSpace")]);
        assert_eq!(config.keys_exit, [key_with("c", &[Modifier::Control])]);
    }

    #[test]
    fn back_and_exit_keys_may_be_given_as_a_list() {
        let config = parse_valid(
            "
            keys_back: [BackSpace, Left]
            keys_exit: [Escape, { key: c, modifiers: Control }]
            ",
        );
        assert_eq!(config.keys_back, [key("BackSpace"), key("Left")]);
        assert_eq!(
            config.keys_exit,
            [key("Escape"), key_with("c", &[Modifier::Control])]
        );
    }

    #[test]
    fn an_empty_exit_key_list_leaves_no_exit_key() {
        assert_eq!(parse_valid("keys_exit: []").keys_exit, []);
    }

    #[test]
    fn steps_run_in_order_and_synchronously_by_default() {
        let config = parse_valid(
            "
            commands:
              - name: Greet
                shortcut: g
                steps:
                  - bash: echo hello
                  - bash: echo world
            ",
        );
        assert_eq!(
            programs(command(&config.root_layer, "g")),
            [
                ("echo hello".to_string(), true),
                ("echo world".to_string(), true)
            ]
        );
    }

    #[test]
    fn steps_run_in_the_background_when_the_command_is_not_synchronous() {
        let config = parse_valid(
            "
            commands:
              - name: Greet
                shortcut: g
                steps:
                  - bash: echo hello
                synchronous: false
            ",
        );
        assert_eq!(
            programs(command(&config.root_layer, "g")),
            [("echo hello".to_string(), false)]
        );
    }

    #[test]
    fn a_step_with_an_empty_program_is_rejected() {
        let error = rejection(&root_parameters("[{ bash: '' }]", "[]"));
        assert!(
            error.contains("program must not be an empty string"),
            "{error}"
        );
    }

    #[test]
    fn a_command_without_steps_is_rejected() {
        let error = rejection(&root_parameters("[]", "[]"));
        assert!(error.contains("Command has no associated steps"), "{error}");
    }

    #[test]
    fn a_placeholder_that_is_not_a_parameter_index_is_rejected() {
        let error = rejection(&root_parameters("[{ bash: 'echo {name}' }]", "[]"));
        assert!(
            error.contains("name is not a valid parameter index"),
            "{error}"
        );
    }

    #[test]
    fn a_placeholder_without_a_declared_parameter_is_rejected() {
        let error = rejection(&root_parameters("[{ bash: 'echo {0}' }]", "[]"));
        assert!(
            error.contains("required 0. parameter was not declared"),
            "{error}"
        );
    }

    #[test]
    fn a_declared_parameter_that_no_step_uses_is_rejected() {
        let error = rejection(&root_parameters(
            "[{ bash: 'echo hello' }]",
            "[{ name: Name, type: text }]",
        ));
        assert!(error.contains("declared 0. parameter is unused"), "{error}");
    }

    #[test]
    fn a_text_parameter_is_declared_by_its_type() {
        let config = parse_valid(&root_parameters(
            "[{ bash: 'echo {0}' }]",
            "[{ name: Name, type: text }]",
        ));
        let command = command(&config.root_layer, "g");
        assert_eq!(parameter_names(command), ["Name"]);
        assert!(matches!(
            command.get_parameters().as_slice(),
            [CommandParameter {
                parameter: ParameterDefinitionVariant::Text(_),
                ..
            }]
        ));
    }

    #[test]
    fn a_character_parameter_is_declared_by_its_type() {
        let config = parse_valid(&root_parameters(
            "[{ bash: 'echo {0}' }]",
            "[{ name: Letter, type: character }]",
        ));
        let command = command(&config.root_layer, "g");
        assert_eq!(parameter_names(command), ["Letter"]);
        assert!(matches!(
            command.get_parameters().as_slice(),
            [CommandParameter {
                parameter: ParameterDefinitionVariant::Character(_),
                ..
            }]
        ));
    }

    #[test]
    fn a_single_parameter_may_be_given_without_a_list() {
        let config = parse_valid(&root_parameters(
            "[{ bash: 'echo {0}' }]",
            "{ name: Name, type: text }",
        ));
        assert_eq!(parameter_names(command(&config.root_layer, "g")), ["Name"]);
    }

    #[test]
    fn parameters_are_indexed_in_declaration_order() {
        let config = parse_valid(&root_parameters(
            "[{ bash: 'echo {1} {0}' }]",
            "[{ name: First, type: text }, { name: Second, type: character }]",
        ));
        assert_eq!(
            parameter_names(command(&config.root_layer, "g")),
            ["First", "Second"]
        );
    }

    #[test]
    fn a_choose_parameter_offers_its_preconfigured_options() {
        let config = parse_valid(&root_parameters(
            "[{ bash: 'echo {0}' }]",
            "[{ name: Fruit, type: choose, options: [apple, banana] }]",
        ));
        let choose = only_choose_parameter(command(&config.root_layer, "g"));
        assert_eq!(choose.options, ["apple", "banana"]);
        assert_eq!(choose.gen_options_command, None);
    }

    #[test]
    fn a_choose_parameter_generates_options_one_per_line_by_default() {
        let config = parse_valid(&root_parameters(
            "[{ bash: 'echo {0}' }]",
            "[{ name: File, type: choose, generate_options: { command: ls } }]",
        ));
        let choose = only_choose_parameter(command(&config.root_layer, "g"));
        assert!(choose.options.is_empty());
        assert_eq!(choose.gen_options_command.as_deref(), Some("ls"));
        assert_eq!(choose.gen_options_split_by, "\n");
    }

    #[test]
    fn a_choose_parameter_splits_generated_options_by_a_configured_separator() {
        let config = parse_valid(&root_parameters(
            "[{ bash: 'echo {0}' }]",
            "[{ name: Path, type: choose, generate_options: { command: 'echo $PATH', split_by: ':' } }]",
        ));
        let choose = only_choose_parameter(command(&config.root_layer, "g"));
        assert_eq!(choose.gen_options_command.as_deref(), Some("echo $PATH"));
        assert_eq!(choose.gen_options_split_by, ":");
    }

    #[test]
    fn a_choose_parameter_without_options_or_a_generating_command_is_rejected() {
        let error = rejection(&root_parameters(
            "[{ bash: 'echo {0}' }]",
            "[{ name: Fruit, type: choose }]",
        ));
        assert!(
            error.contains("Invalid 'choose' parameter configuration"),
            "{error}"
        );
    }

    #[test]
    fn a_parameter_of_an_unsupported_type_is_rejected() {
        let error = rejection(&root_parameters(
            "[{ bash: 'echo {0}' }]",
            "[{ name: Count, type: number }]",
        ));
        assert!(
            error.contains("parameter type number is unsupported"),
            "{error}"
        );
    }

    #[test]
    fn nested_layers_hold_their_commands_and_layers_under_their_shortcuts() {
        let config = parse_valid(
            "
            layers:
              - name: Applications
                shortcut: a
                layers:
                  - name: Browsers
                    shortcut: b
                    commands:
                      - name: Firefox
                        shortcut: f
                        steps:
                          - bash: firefox
                commands:
                  - name: Terminal
                    shortcut: t
                    steps:
                      - bash: kitty
            ",
        );
        let applications = sublayer(&config.root_layer, "a");
        assert_eq!(applications.name, "Applications");
        assert_eq!(shortcuts(applications), [key("b"), key("t")]);
        assert_eq!(command(applications, "t").name, "Terminal");
        let browsers = sublayer(applications, "b");
        assert_eq!(browsers.name, "Browsers");
        assert_eq!(command(browsers, "f").name, "Firefox");
    }

    #[test]
    fn the_same_shortcut_may_be_used_in_different_layers() {
        let config = parse_valid(
            "
            layers:
              - name: Applications
                shortcut: a
                commands:
                  - name: Alacritty
                    shortcut: a
                    steps:
                      - bash: alacritty
            ",
        );
        assert_eq!(
            command(sublayer(&config.root_layer, "a"), "a").name,
            "Alacritty"
        );
    }

    #[test]
    fn two_commands_on_the_same_shortcut_in_a_layer_are_rejected() {
        let error = rejection(
            "
            commands:
              - name: Greet
                shortcut: g
                steps:
                  - bash: echo hello
              - name: Goodbye
                shortcut: g
                steps:
                  - bash: echo goodbye
            ",
        );
        assert!(
            error.contains("Could not assign command to key g because of an existing binding"),
            "{error}"
        );
    }

    #[test]
    fn two_layers_on_the_same_shortcut_in_a_layer_are_rejected() {
        let error = rejection(
            "
            layers:
              - name: Applications
                shortcut: a
              - name: Audio
                shortcut: a
            ",
        );
        assert!(
            error.contains("Can not assign layer to key a because of an existing binding"),
            "{error}"
        );
    }

    #[test]
    fn a_command_on_the_shortcut_of_a_layer_beside_it_is_rejected() {
        let error = rejection(
            "
            layers:
              - name: Applications
                shortcut: a
            commands:
              - name: Alacritty
                shortcut: a
                steps:
                  - bash: alacritty
            ",
        );
        assert!(
            error.contains("Could not assign command to key a because of an existing binding"),
            "{error}"
        );
    }

    #[test]
    fn two_spellings_of_one_keysym_in_a_layer_are_the_same_shortcut() {
        let error = rejection(
            "
            commands:
              - name: Up
                shortcut: Page_Up
                steps:
                  - bash: echo up
              - name: Previous
                shortcut: Prior
                steps:
                  - bash: echo previous
            ",
        );
        assert!(
            error.contains("Could not assign command to key Prior because of an existing binding"),
            "{error}"
        );
    }

    #[test]
    fn a_root_shortcut_on_an_exit_key_is_rejected() {
        let error = rejection(&root_shortcut("Escape"));
        assert!(
            error.contains(
                "Shortcut Escape in layer \"Root\" can never be pressed because it is consumed by \
                 keys_exit"
            ),
            "{error}"
        );
    }

    #[test]
    fn a_layer_shortcut_on_an_exit_key_is_rejected() {
        let error = rejection(
            "
            keys_exit: q
            layers:
              - name: Quit
                shortcut: q
            ",
        );
        assert!(error.contains("Shortcut q in layer \"Root\""), "{error}");
        assert!(error.contains("consumed by keys_exit"), "{error}");
    }

    #[test]
    fn a_nested_shortcut_on_an_exit_key_is_rejected() {
        let error = rejection(
            "
            layers:
              - name: Applications
                shortcut: a
                commands:
                  - name: Escape
                    shortcut: Escape
                    steps:
                      - bash: echo escape
            ",
        );
        assert!(
            error.contains("Shortcut Escape in layer \"Applications\""),
            "{error}"
        );
        assert!(error.contains("consumed by keys_exit"), "{error}");
    }

    #[test]
    fn a_root_shortcut_on_a_back_key_is_allowed() {
        let config = parse_valid(
            "
            keys_back: BackSpace
            layers:
              - name: Back
                shortcut: BackSpace
            ",
        );
        assert_eq!(sublayer(&config.root_layer, "BackSpace").name, "Back");
    }

    #[test]
    fn a_nested_shortcut_on_a_back_key_is_rejected() {
        let error = rejection(
            "
            keys_back: BackSpace
            layers:
              - name: Applications
                shortcut: a
                layers:
                  - name: Back
                    shortcut: BackSpace
            ",
        );
        assert!(
            error.contains("Shortcut BackSpace in layer \"Applications\""),
            "{error}"
        );
        assert!(error.contains("consumed by keys_back"), "{error}");
    }

    #[test]
    fn a_shortcut_differing_from_an_exit_key_by_its_modifiers_is_allowed() {
        let config = parse_valid(&root_shortcut("{ key: Escape, modifiers: Shift }"));
        assert_eq!(
            shortcuts(&config.root_layer),
            [key_with("Escape", &[Modifier::Shift])]
        );
    }

    #[test]
    fn an_application_layer_applies_to_windows_its_pattern_matches() {
        let config = parse_valid(
            "
            app_specific_layers:
              - match: '.*[Ff]irefox.*'
                commands:
                  - name: Open DevTools
                    shortcut: d
                    steps:
                      - bash: xdotool key F12
            ",
        );
        let [application] = config.app_specific_layers.as_slice() else {
            panic!("there is not exactly one application layer");
        };
        assert!(application.pattern.is_match("Mozilla Firefox"));
        assert!(!application.pattern.is_match("kitty"));
        assert_eq!(application.root_layer.name, ".*[Ff]irefox.*");
        assert_eq!(command(&application.root_layer, "d").name, "Open DevTools");
    }

    #[test]
    fn application_layers_keep_their_configured_order() {
        let config = parse_valid(
            "
            app_specific_layers:
              - match: firefox
              - match: kitty
            ",
        );
        let patterns: Vec<&str> = config
            .app_specific_layers
            .iter()
            .map(|application| application.pattern.as_str())
            .collect();
        assert_eq!(patterns, ["firefox", "kitty"]);
    }

    #[test]
    fn an_application_layer_with_an_invalid_pattern_is_rejected() {
        let error = rejection(
            "
            app_specific_layers:
              - match: '('
            ",
        );
        assert!(error.contains("Invalid regex pattern \"(\""), "{error}");
    }

    #[test]
    fn an_application_layer_with_a_duplicate_shortcut_is_rejected() {
        let error = rejection(
            "
            app_specific_layers:
              - match: firefox
                layers:
                  - name: Tabs
                    shortcut: t
                  - name: Toolbars
                    shortcut: t
            ",
        );
        assert!(
            error.contains("Can not assign layer to key t because of an existing binding"),
            "{error}"
        );
    }

    #[test]
    fn an_application_layer_shortcut_on_an_exit_key_is_rejected() {
        let error = rejection(
            "
            app_specific_layers:
              - match: firefox
                commands:
                  - name: Escape
                    shortcut: Escape
                    steps:
                      - bash: echo escape
            ",
        );
        assert!(
            error.contains("Shortcut Escape in layer \"firefox\""),
            "{error}"
        );
        assert!(error.contains("consumed by keys_exit"), "{error}");
    }
}
