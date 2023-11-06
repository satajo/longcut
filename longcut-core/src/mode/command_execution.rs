use crate::mode::error::{ErrorMode, ErrorResult};
use crate::mode::parameter_input::{
    ParameterInputContext, ParameterInputMode, ParameterInputResult,
};
use crate::model::command::{Command, Instruction};
use crate::model::layer::Layer;
use crate::model::parameter::ParameterValueVariant;
use crate::port::executor::Executor;

/// Orchestrates the user-requested command executions.
pub struct CommandExecutionMode<'a> {
    executor: &'a dyn Executor,
    // Configuration
    error_mode: &'a ErrorMode<'a>,
    parameter_input_mode: &'a ParameterInputMode<'a>,
}

pub enum CommandExecutionResult {
    Finished,
    KeepGoing,
}

impl<'a> CommandExecutionMode<'a> {
    pub fn new(
        executor: &'a dyn Executor,
        error_mode: &'a ErrorMode<'a>,
        parameter_input_mode: &'a ParameterInputMode<'a>,
    ) -> Self {
        Self {
            executor,
            error_mode,
            parameter_input_mode,
        }
    }

    pub fn run(&self, command: &Command, layers: &[&Layer]) -> CommandExecutionResult {
        // Values for all parameters required for the execution are read.
        let parameter_values = match self.read_parameter_values(command, layers) {
            Ok(parameters) => parameters,
            Err(result) => {
                return result;
            }
        };

        // With the parameters read, the command template is rendered using them.
        // An error here is considered irrecoverable, indicating a flaw in the program itself.
        let instructions = command
            .render_instructions(parameter_values)
            .expect("Internal logic error: Debug command execution program behaviour");

        // The instructions are executed one after another. On error the user may choose
        // to abort the execution so we return the chosen result as is.
        for instruction in instructions {
            match self.execute_program_instruction(instruction) {
                Ok(_) => {}
                Err(error) => {
                    return error;
                }
            }
        }

        // All instructions have been executed successfully. Depending on the command we either
        // instruct to terminate the sequence or to keep going, enabling the user to rapidly re-
        // trigger the same or some other command.
        match command.is_final {
            true => CommandExecutionResult::Finished,
            false => CommandExecutionResult::KeepGoing,
        }
    }

    fn read_parameter_values(
        &self,
        command: &Command,
        layers: &[&Layer],
    ) -> Result<Vec<ParameterValueVariant>, CommandExecutionResult> {
        let context = ParameterInputContext { command, layers };

        let mut values: Vec<ParameterValueVariant> = vec![];

        // Parameters values are read one by one into a vector using the parameter input mode..
        for parameter in command.get_parameters() {
            let parameter_value = match self.parameter_input_mode.run(&context, parameter) {
                ParameterInputResult::Ok(value) => value,
                ParameterInputResult::Cancel => {
                    return Err(CommandExecutionResult::KeepGoing);
                }
                ParameterInputResult::Exit => {
                    return Err(CommandExecutionResult::Finished);
                }
            };

            values.push(parameter_value);
        }
        Ok(values)
    }

    fn execute_program_instruction(
        &self,
        instruction: Instruction,
    ) -> Result<(), CommandExecutionResult> {
        // Execution happens in a loop to facilitate retry on failure.
        loop {
            let result = if instruction.is_synchronous {
                self.executor
                    .run_to_completion(&instruction.program_string)
                    .map(|_| ())
            } else {
                self.executor.run_in_background(&instruction.program_string)
            };

            let Err(error) = result else {
                // On success we're done and return right away.
                return Ok(());
            };

            // On error the error data is passed onto the error handling program, letting
            // the user decide what to do next.
            match self.error_mode.run(&error) {
                ErrorResult::Abort => {
                    return Err(CommandExecutionResult::Finished);
                }
                ErrorResult::Cancel => {
                    return Err(CommandExecutionResult::KeepGoing);
                }
                ErrorResult::Retry => {
                    println!("Retrying execution! {:?}", error);
                }
            }
        }
    }
}
