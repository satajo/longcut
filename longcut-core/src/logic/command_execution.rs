use crate::logic::error::{ErrorProgram, ProgramResult as ErrorProgramResult};
use crate::logic::parameter_input::{
    ParameterInputProgram, ProgramContext as ParameterInputProgramContext,
    ProgramResult as ParameterInputProgramResult,
};
use crate::model::command::{Command, CommandParameter, Instruction};
use crate::model::layer::Layer;
use crate::model::parameter::ParameterValue;
use crate::port::executor::Executor;

pub struct CommandExecutionProgram<'a> {
    executor: &'a dyn Executor,
    // Configuration
    error_program: &'a ErrorProgram<'a>,
    parameter_input_program: &'a ParameterInputProgram<'a>,
}

pub enum ProgramResult {
    Finished,
    KeepGoing,
}

impl<'a> CommandExecutionProgram<'a> {
    pub fn new(
        executor: &'a dyn Executor,
        error_program: &'a ErrorProgram<'a>,
        parameter_input_program: &'a ParameterInputProgram<'a>,
    ) -> Self {
        Self {
            executor,
            error_program,
            parameter_input_program,
        }
    }

    pub fn run(&self, command: &Command, layers: &[&Layer]) -> ProgramResult {
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
            .render_instructions(&parameter_values)
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
            true => ProgramResult::Finished,
            false => ProgramResult::KeepGoing,
        }
    }

    fn read_parameter_values(
        &self,
        command: &Command,
        layers: &[&Layer],
    ) -> Result<Vec<ParameterValue>, ProgramResult> {
        let context = ParameterInputProgramContext { command, layers };

        // Parameters values are read one by one into a vector using the subroutine.
        let mut values: Vec<ParameterValue> = vec![];
        for parameter in command.get_parameters() {
            values.push(self.read_parameter_value(&context, parameter)?);
        }
        Ok(values)
    }

    fn read_parameter_value(
        &self,
        context: &ParameterInputProgramContext,
        parameter: &CommandParameter,
    ) -> Result<ParameterValue, ProgramResult> {
        match self.parameter_input_program.run(context, parameter) {
            ParameterInputProgramResult::Ok(value) => Ok(value),
            ParameterInputProgramResult::Cancel => Err(ProgramResult::KeepGoing),
            ParameterInputProgramResult::Exit => Err(ProgramResult::Finished),
        }
    }

    fn execute_program_instruction(&self, instruction: Instruction) -> Result<(), ProgramResult> {
        // Execution happens in a loop to facilitate retry on failure.
        loop {
            match self.executor.execute(&instruction) {
                // On success we're done and return right away.
                Ok(_) => {
                    return Ok(());
                }
                // On error the error data is passed onto the error handling program, letting
                // the user decide what to do next.
                Err(error) => match self.error_program.run(&error) {
                    ErrorProgramResult::Abort => {
                        return Err(ProgramResult::Finished);
                    }
                    ErrorProgramResult::Cancel => {
                        return Err(ProgramResult::KeepGoing);
                    }
                    ErrorProgramResult::Retry => {
                        println!("Retrying execution! {:?}", error);
                    }
                },
            }
        }
    }
}
