use crate::model::command::{Command, ParameterDeclaration, ParameterValue};
use crate::model::key::{Key, Symbol};
use crate::port::executor::Executor;
use crate::port::input::Input;
use crate::port::view::{View, ViewState};

pub struct CommandExecutionProgram<'a> {
    executor: &'a dyn Executor,
    input: &'a dyn Input,
    view: &'a dyn View,
    keys_deactivate: &'a [Key],
}

pub enum ProgramResult {
    Finished,
    KeepGoing,
}

impl<'a> CommandExecutionProgram<'a> {
    pub fn new(
        executor: &'a impl Executor,
        input: &'a impl Input,
        view: &'a impl View,
        keys_deactivate: &'a [Key],
    ) -> Self {
        Self {
            executor,
            input,
            view,
            keys_deactivate,
        }
    }

    pub fn run(&self, command: &Command) -> ProgramResult {
        let mut parameters: Vec<ParameterValue> = vec![];
        for declaration in command.get_parameters() {
            match declaration {
                // Character parameter handling
                ParameterDeclaration::Character => match self.resolve_character_parameter() {
                    ReadParameterResult::Ok(value) => {
                        parameters.push(ParameterValue::Character(value));
                    }
                    ReadParameterResult::Cancel => return ProgramResult::KeepGoing,
                    ReadParameterResult::Exit => return ProgramResult::Finished,
                },

                // Text parameter handling
                ParameterDeclaration::Text => match self.read_text_parameter() {
                    ReadParameterResult::Ok(value) => {
                        parameters.push(ParameterValue::Text(value));
                    }
                    ReadParameterResult::Cancel => return ProgramResult::KeepGoing,
                    ReadParameterResult::Exit => return ProgramResult::Finished,
                },
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

    fn resolve_character_parameter(&self) -> ReadParameterResult<char> {
        self.view.render(&ViewState::InputCharacter);
        loop {
            let press = self.input.capture_any();
            if self.keys_deactivate.contains(&press) {
                return ReadParameterResult::Exit;
            }

            match press.symbol {
                Symbol::Character(c) => return ReadParameterResult::Ok(c),
                Symbol::BackSpace => return ReadParameterResult::Cancel,
                _ => println!("Not a character!"),
            }
        }
    }

    fn read_text_parameter(&self) -> ReadParameterResult<String> {
        let mut input = String::new();
        loop {
            self.view.render(&ViewState::InputString {
                input: input.clone(),
            });

            let press = self.input.capture_any();
            if self.keys_deactivate.contains(&press) {
                return ReadParameterResult::Exit;
            }

            match press.symbol {
                Symbol::Character(c) => input.push(c),
                Symbol::Return => {
                    return ReadParameterResult::Ok(input);
                }
                Symbol::BackSpace => {
                    if !input.is_empty() {
                        input.pop();
                    } else {
                        return ReadParameterResult::Cancel;
                    }
                }
                _ => { /* Irrelevant input. */ }
            }
        }
    }
}

enum ReadParameterResult<T> {
    Ok(T),
    Cancel,
    Exit,
}
