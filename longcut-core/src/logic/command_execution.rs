use crate::logic::error::{ErrorProgram, ProgramResult as ErrorProgramResult};
use crate::model::command::{Command, CommandParameter};
use crate::model::key::{Key, Symbol};
use crate::model::layer::Layer;
use crate::model::parameter::{Parameter, ParameterValue};
use crate::port::executor::Executor;
use crate::port::input::Input;
use crate::port::view::{ParameterInputViewModel, View, ViewModel};

pub struct CommandExecutionProgram<'a> {
    executor: &'a dyn Executor,
    input: &'a dyn Input,
    view: &'a dyn View,
    // Configuration
    error_program: &'a ErrorProgram<'a>,
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
        error_program: &'a ErrorProgram<'a>,
        keys_deactivate: &'a [Key],
    ) -> Self {
        Self {
            executor,
            input,
            view,
            error_program,
            keys_deactivate,
        }
    }

    pub fn run(&self, command: &Command, layers: &[&Layer]) -> ProgramResult {
        let mut parameters: Vec<ParameterValue> = vec![];
        for declaration in command.get_parameters() {
            match declaration.parameter {
                // Character parameter handling
                Parameter::Character => {
                    match self.read_character_parameter(declaration, command, layers) {
                        ReadParameterResult::Ok(value) => {
                            parameters.push(ParameterValue::Character(value));
                        }
                        ReadParameterResult::Cancel => return ProgramResult::KeepGoing,
                        ReadParameterResult::Exit => return ProgramResult::Finished,
                    }
                }

                // Text parameter handling
                Parameter::Text => match self.read_text_parameter(declaration, command, layers) {
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
            loop {
                // Executed happens in a loop to facilitate retry on failure.
                match self.executor.execute(&instruction) {
                    Ok(_) => break,
                    Err(error) => match self.error_program.run(&error) {
                        ErrorProgramResult::Abort => {
                            return ProgramResult::Finished;
                        }
                        ErrorProgramResult::Cancel => {
                            return ProgramResult::KeepGoing;
                        }
                        ErrorProgramResult::Retry => {
                            println!("Retrying execution! {:?}", error);
                        }
                    },
                }
            }
        }

        match command.is_final {
            true => ProgramResult::Finished,
            false => ProgramResult::KeepGoing,
        }
    }

    fn read_character_parameter(
        &self,
        parameter: &CommandParameter,
        command: &Command,
        layers: &[&Layer],
    ) -> ReadParameterResult<char> {
        self.render(parameter, command, layers, "");

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

    fn read_text_parameter(
        &self,
        parameter: &CommandParameter,
        command: &Command,
        layers: &[&Layer],
    ) -> ReadParameterResult<String> {
        let mut input = String::new();
        loop {
            self.render(parameter, command, layers, &input);

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

    fn render(
        &self,
        parameter: &CommandParameter,
        command: &Command,
        layers: &[&Layer],
        input_value: &str,
    ) {
        let state = ViewModel::ParameterInput(ParameterInputViewModel {
            command,
            input_value,
            layers,
            parameter,
        });

        self.view.render(state);
    }
}

enum ReadParameterResult<T> {
    Ok(T),
    Cancel,
    Exit,
}
