use super::Context;
use super::error::{ErrorResult, run_error_mode};
use super::parameter_input::{
    ParameterInputContext, ParameterInputResult, run_parameter_input_mode,
};
use crate::model::command::{Command, Instruction};
use crate::model::layer::Layer;
use crate::model::parameter::ParameterValueVariant;

pub enum CommandExecutionResult {
    Finished,
    KeepGoing,
}

/// Orchestrates the user-requested command executions.
pub fn run_command_execution_mode(
    ctx: &Context,
    command: &Command,
    layers: &[&Layer],
) -> CommandExecutionResult {
    // Values for all parameters required for the execution are read.
    let parameter_values = match read_parameter_values(ctx, command, layers) {
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
        match execute_program_instruction(ctx, instruction) {
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
    ctx: &Context,
    command: &Command,
    layers: &[&Layer],
) -> Result<Vec<ParameterValueVariant>, CommandExecutionResult> {
    let p_input_context = ParameterInputContext { command, layers };

    let mut values: Vec<ParameterValueVariant> = vec![];

    // Parameters values are read one by one into a vector using the parameter input mode..
    for parameter in command.get_parameters() {
        let parameter_value = match run_parameter_input_mode(ctx, &p_input_context, parameter) {
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
    ctx: &Context,
    instruction: Instruction,
) -> Result<(), CommandExecutionResult> {
    // Execution happens in a loop to facilitate retry on failure.
    loop {
        let result = if instruction.is_synchronous {
            ctx.executor
                .run_to_completion(&instruction.program_string)
                .map(|_| ())
        } else {
            ctx.executor.run_in_background(&instruction.program_string)
        };

        let Err(error) = result else {
            // On success we're done and return right away.
            return Ok(());
        };

        // On error the error data is passed onto the error handling program, letting
        // the user decide what to do next.
        match run_error_mode(ctx, &error) {
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
