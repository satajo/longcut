use crate::model::command::{Command, CommandParameter};
use crate::model::key::{Key, Symbol};
use crate::model::layer::Layer;
use crate::model::parameter::{Parameter, ParameterValue};
use crate::model::shortcut_map::ShortcutMap;
use crate::port::input::Input;
use crate::port::view::{ParameterInputViewModel, ViewModel};
use crate::port::view::{ParameterVariant, View};

pub struct ParameterInputProgram<'a> {
    input: &'a dyn Input,
    view: &'a dyn View,
    keys_back: &'a [Key],
    keys_deactivate: &'a [Key],
}

pub enum ProgramResult {
    Ok(ParameterValue),
    Cancel,
    Exit,
}

pub struct ProgramContext<'a> {
    pub command: &'a Command,
    pub layers: &'a [&'a Layer],
}

impl<'a> ParameterInputProgram<'a> {
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

    pub fn run(&self, context: &ProgramContext, parameter: &CommandParameter) -> ProgramResult {
        match &parameter.parameter {
            Parameter::Character => self.read_character_parameter(context, parameter),
            Parameter::Text => self.read_text_parameter(context, parameter),
            Parameter::Choose(options) => self.read_choose_parameter(context, parameter, options),
        }
    }

    fn read_character_parameter(
        &self,
        context: &ProgramContext,
        parameter: &CommandParameter,
    ) -> ProgramResult {
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
                return ProgramResult::Exit;
            }

            if self.keys_back.contains(&press) {
                return ProgramResult::Cancel;
            }

            match press.symbol {
                Symbol::Character(c) => {
                    let value = ParameterValue::Character(c);
                    return ProgramResult::Ok(value);
                }
                _ => { /* Irrelevant input. */ }
            }
        }
    }

    fn read_text_parameter(
        &self,
        context: &ProgramContext,
        parameter: &CommandParameter,
    ) -> ProgramResult {
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
                return ProgramResult::Exit;
            }

            if self.keys_back.contains(&press) && input.is_empty() {
                return ProgramResult::Cancel;
            }

            match press.symbol {
                Symbol::Character(c) => input.push(c),
                Symbol::Return => {
                    let value = ParameterValue::Text(input);
                    return ProgramResult::Ok(value);
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
        context: &ProgramContext,
        parameter: &CommandParameter,
        options: &[String],
    ) -> ProgramResult {
        let mut shortcuts = ShortcutMap::<String>::new();

        // The options are indexed into a ShortcutMap by their first letter.
        {
            for option in options {
                let Some(first_char) = option.chars().next() else {
                    // Strange, the option does not appear to have a name.
                    continue;
                };
                let key = Key::new(Symbol::Character(first_char.to_ascii_lowercase()));

                // TODO: Handle duplicates sensibly.
                let _ = shortcuts.try_assign(key, option.to_owned());
            }
        }

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
                return ProgramResult::Exit;
            }

            if self.keys_back.contains(&press) {
                return ProgramResult::Cancel;
            }

            let Some(option) = shortcuts.get(&press) else {
                continue;
            };

            let value = ParameterValue::Choice(option.to_string());
            return ProgramResult::Ok(value);
        }
    }
}
