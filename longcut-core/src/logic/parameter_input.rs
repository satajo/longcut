use crate::model::command::CommandParameter;
use crate::model::parameter::{Parameter, ParameterValue};
use crate::port::view::{ParameterInputViewModel, ViewModel};
use crate::{Input, Key, Symbol, View};

pub struct ParameterInputProgram<'a> {
    input: &'a dyn Input,
    view: &'a dyn View,
    keys_deactivate: &'a [Key],
}

pub enum ParameterInputProgramResult {
    Ok(ParameterValue),
    Cancel,
    Exit,
}

impl<'a> ParameterInputProgram<'a> {
    pub fn new(input: &'a impl Input, view: &'a impl View, keys_deactivate: &'a [Key]) -> Self {
        Self {
            input,
            view,
            keys_deactivate,
        }
    }

    pub fn run(
        &self,
        context: &[&str],
        parameter: &CommandParameter,
    ) -> ParameterInputProgramResult {
        match parameter.parameter {
            Parameter::Character => self.read_character_parameter(parameter, context),
            Parameter::Text => self.read_text_parameter(parameter, context),
        }
    }

    fn read_character_parameter(
        &self,
        parameter: &CommandParameter,
        context: &[&str],
    ) -> ParameterInputProgramResult {
        self.render(parameter, context, "");

        loop {
            let press = self.input.capture_any();

            if self.keys_deactivate.contains(&press) {
                return ParameterInputProgramResult::Exit;
            }

            match press.symbol {
                Symbol::Character(c) => {
                    let value = ParameterValue::Character(c);
                    return ParameterInputProgramResult::Ok(value);
                }
                Symbol::BackSpace => return ParameterInputProgramResult::Cancel,
                _ => { /* Irrelevant input. */ }
            }
        }
    }

    fn read_text_parameter(
        &self,
        parameter: &CommandParameter,
        context: &[&str],
    ) -> ParameterInputProgramResult {
        let mut input = String::new();
        loop {
            self.render(parameter, context, &input);

            let press = self.input.capture_any();

            if self.keys_deactivate.contains(&press) {
                return ParameterInputProgramResult::Exit;
            }

            match press.symbol {
                Symbol::Character(c) => input.push(c),
                Symbol::Return => {
                    let value = ParameterValue::Text(input);
                    return ParameterInputProgramResult::Ok(value);
                }
                Symbol::BackSpace => {
                    if !input.is_empty() {
                        input.pop();
                    } else {
                        return ParameterInputProgramResult::Cancel;
                    }
                }
                _ => { /* Irrelevant input. */ }
            }
        }
    }

    fn render(&self, parameter: &CommandParameter, context: &[&str], input_value: &str) {
        let model = ParameterInputViewModel {
            parameter,
            context,
            input_value,
        };

        self.view.render(ViewModel::ParameterInput(model));
    }
}
