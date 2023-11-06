use crate::model::command::{Command, CommandParameter};
use crate::model::key::{Key, Symbol};
use crate::model::layer::Layer;
use crate::model::parameter::{Parameter, ParameterValue};
use crate::model::shortcut_map::ShortcutMap;
use crate::port::input::Input;
use crate::port::view::{ParameterInputViewModel, ViewModel};
use crate::port::view::{ParameterVariant, View};

/// Processes input from the user to generate values for command parameters.
pub struct ParameterInputMode<'a> {
    input: &'a dyn Input,
    view: &'a dyn View,
    keys_back: &'a [Key],
    keys_deactivate: &'a [Key],
}

pub enum ParameterInputResult {
    Ok(ParameterValue),
    Cancel,
    Exit,
}

pub struct ParameterInputContext<'a> {
    pub command: &'a Command,
    pub layers: &'a [&'a Layer],
}

impl<'a> ParameterInputMode<'a> {
    pub fn new(
        input: &'a dyn Input,
        view: &'a dyn View,
        keys_back: &'a [Key],
        keys_deactivate: &'a [Key],
    ) -> Self {
        Self {
            input,
            view,
            keys_back,
            keys_deactivate,
        }
    }

    pub fn run(
        &self,
        context: &ParameterInputContext,
        parameter: &CommandParameter,
    ) -> ParameterInputResult {
        match &parameter.parameter {
            Parameter::Character => self.read_character_parameter(context, parameter),
            Parameter::Text => self.read_text_parameter(context, parameter),
            Parameter::Choose(options) => self.read_choose_parameter(context, parameter, options),
        }
    }

    fn read_character_parameter(
        &self,
        context: &ParameterInputContext,
        parameter: &CommandParameter,
    ) -> ParameterInputResult {
        let view_model = ParameterInputViewModel {
            command: context.command,
            parameter_name: parameter.name.as_str(),
            parameter: ParameterVariant::CharInput,
            layer_stack: context.layers,
        };
        self.view.render(ViewModel::ParameterInput(view_model));

        loop {
            let press = self.input.capture_any();

            if self.keys_deactivate.contains(&press) {
                return ParameterInputResult::Exit;
            }

            if self.keys_back.contains(&press) {
                return ParameterInputResult::Cancel;
            }

            match press.symbol {
                Symbol::Character(c) => {
                    let value = ParameterValue::Character(c);
                    return ParameterInputResult::Ok(value);
                }
                _ => { /* Irrelevant input. */ }
            }
        }
    }

    fn read_text_parameter(
        &self,
        context: &ParameterInputContext,
        parameter: &CommandParameter,
    ) -> ParameterInputResult {
        let mut input = String::new();
        loop {
            let view_model = ParameterInputViewModel {
                command: context.command,
                parameter_name: parameter.name.as_str(),
                parameter: ParameterVariant::StringInput {
                    input_value: &input,
                },
                layer_stack: context.layers,
            };
            self.view.render(ViewModel::ParameterInput(view_model));

            let press = self.input.capture_any();

            if self.keys_deactivate.contains(&press) {
                return ParameterInputResult::Exit;
            }

            if self.keys_back.contains(&press) && input.is_empty() {
                return ParameterInputResult::Cancel;
            }

            match press.symbol {
                Symbol::Character(c) => input.push(c),
                Symbol::Return => {
                    let value = ParameterValue::Text(input);
                    return ParameterInputResult::Ok(value);
                }
                Symbol::BackSpace => {
                    input.pop();
                }
                _ => { /* Irrelevant input. */ }
            }
        }
    }

    fn read_choose_parameter(
        &self,
        context: &ParameterInputContext,
        parameter: &CommandParameter,
        options: &[String],
    ) -> ParameterInputResult {
        let mut shortcuts = ShortcutMap::<&String>::new();
        let options_as_mnemonic_pairs = options
            .iter()
            .map(|option| (option.as_str(), option))
            .collect();
        shortcuts.auto_assign_mnemonics(options_as_mnemonic_pairs);

        // The view is rendered based on the shortcut map content.
        {
            let values: Vec<(&Key, &str)> = shortcuts
                .iter()
                .map(|(key, value)| (key, value.as_str()))
                .collect();

            let view_model = ParameterInputViewModel {
                command: context.command,
                parameter_name: parameter.name.as_str(),
                parameter: ParameterVariant::OptionInput { options: &values },
                layer_stack: context.layers,
            };
            self.view.render(ViewModel::ParameterInput(view_model));
        }

        // With the view render out of the way, we read the input.
        loop {
            let press = self.input.capture_any();

            if self.keys_deactivate.contains(&press) {
                return ParameterInputResult::Exit;
            }

            if self.keys_back.contains(&press) {
                return ParameterInputResult::Cancel;
            }

            let Some(option) = shortcuts.get(&press) else {
                continue;
            };

            let value = ParameterValue::Choice(option.to_string());
            return ParameterInputResult::Ok(value);
        }
    }
}
