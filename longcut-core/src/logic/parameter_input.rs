use crate::model::command::{Command, CommandParameter};
use crate::model::key::{Key, Symbol};
use crate::model::layer::Layer;
use crate::model::parameter::{Parameter, ParameterValue};
use crate::port::input::Input;
use crate::port::view::View;
use crate::port::view::{ParameterInputViewModel, ViewModel};

pub struct ParameterInputProgram<'a> {
    input: &'a dyn Input,
    view: &'a dyn View,
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
    pub fn new(input: &'a dyn Input, view: &'a dyn View, keys_deactivate: &'a [Key]) -> Self {
        Self {
            input,
            view,
            keys_deactivate,
        }
    }

    pub fn run(&self, context: &ProgramContext, parameter: &CommandParameter) -> ProgramResult {
        match parameter.parameter {
            Parameter::Character => self.read_character_parameter(context, parameter),
            Parameter::Text => self.read_text_parameter(context, parameter),
        }
    }

    fn read_character_parameter(
        &self,
        context: &ProgramContext,
        parameter: &CommandParameter,
    ) -> ProgramResult {
        self.render(context, parameter, "");

        loop {
            let press = self.input.capture_any();

            if self.keys_deactivate.contains(&press) {
                return ProgramResult::Exit;
            }

            match press.symbol {
                Symbol::Character(c) => {
                    let value = ParameterValue::Character(c);
                    return ProgramResult::Ok(value);
                }
                Symbol::BackSpace => return ProgramResult::Cancel,
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
            self.render(context, parameter, &input);

            let press = self.input.capture_any();

            if self.keys_deactivate.contains(&press) {
                return ProgramResult::Exit;
            }

            match press.symbol {
                Symbol::Character(c) => input.push(c),
                Symbol::Return => {
                    let value = ParameterValue::Text(input);
                    return ProgramResult::Ok(value);
                }
                Symbol::BackSpace => {
                    if !input.is_empty() {
                        input.pop();
                    } else {
                        return ProgramResult::Cancel;
                    }
                }
                _ => { /* Irrelevant input. */ }
            }
        }
    }

    fn render(&self, context: &ProgramContext, parameter: &CommandParameter, input_value: &str) {
        let model = ParameterInputViewModel {
            command: context.command,
            input_value,
            layer_stack: context.layers,
            parameter,
        };

        self.view.render(ViewModel::ParameterInput(model));
    }
}
