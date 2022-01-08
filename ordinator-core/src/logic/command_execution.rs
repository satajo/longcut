use crate::model::command::{Command, ParameterDeclaration, ParameterValue};
use crate::model::key::Symbol;
use crate::port::executor::Executor;
use crate::port::input::Input;
use crate::port::view::{View, ViewState};

pub struct CommandExecutionProgram<'a> {
    executor: &'a dyn Executor,
    input: &'a dyn Input,
    view: &'a dyn View,
}

pub enum ProgramResult {
    Finished,
    KeepGoing,
}

impl<'a> CommandExecutionProgram<'a> {
    pub fn new(executor: &'a impl Executor, input: &'a impl Input, view: &'a impl View) -> Self {
        Self {
            executor,
            input,
            view,
        }
    }

    pub fn run(&self, command: &Command) -> ProgramResult {
        let mut parameters: Vec<ParameterValue> = vec![];
        for declaration in command.get_parameters() {
            match declaration {
                ParameterDeclaration::Character => {
                    let value = self.resolve_character_parameter();
                    parameters.push(ParameterValue::Character(value));
                }
                ParameterDeclaration::Text => {
                    let value = self.read_string_parameter();
                    parameters.push(ParameterValue::Text(value));
                }
            }
        }

        let instructions = command
            .render_instructions(&parameters)
            .expect("Internal logic error: Debug command execution program behaviour");

        for instruction in instructions {
            if let Err(error) = self.executor.execute(&instruction) {
                println!("Execution failed! {:?}", error)
            }
        }

        match command.is_final {
            true => ProgramResult::Finished,
            false => ProgramResult::KeepGoing,
        }
    }

    fn resolve_character_parameter(&self) -> char {
        self.view.render(&ViewState::InputCharacter);
        loop {
            let press = self.input.capture_any();
            match press.symbol {
                Symbol::Character(c) => return c,
                _ => println!("Not a character!"),
            }
        }
    }

    fn read_string_parameter(&self) -> String {
        let mut input = String::new();
        loop {
            self.view.render(&ViewState::InputString {
                input: input.clone(),
            });

            let press = self.input.capture_any();
            match press.symbol {
                Symbol::Character(c) => input.push(c),
                Symbol::Return => {
                    return input;
                }
                Symbol::BackSpace => {
                    if !input.is_empty() {
                        input.pop();
                    }
                }
                _ => { /* Irrelevant input. */ }
            }
        }
    }
}
