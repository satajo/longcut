use super::Context;
use super::command_running::{RunningResult, run_command_running_mode};
use super::error::{
    ErrorResult, UnrecoverableErrorResult, run_error_mode, run_unrecoverable_error_mode,
};
use super::parameter_input::{
    ParameterInputContext, ParameterInputResult, run_parameter_input_mode,
};
use crate::model::command::Command;
use crate::model::effect::Effect;
use crate::model::layer::Layer;
use crate::model::parameter::ParameterValueVariant;
use crate::port::executor::ExecutorError;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CommandExecutionResult {
    Finished,
    KeepGoing,
}

/// Orchestrates the user-requested command executions.
pub(crate) fn run_command_execution_mode(
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
    let effects = command
        .render_effects(parameter_values)
        .expect("Internal logic error: Debug command execution program behaviour");

    // The effects are executed one after another. On error the user may choose
    // to abort the execution so we return the chosen result as is.
    for effect in effects {
        match execute_effect(ctx, &command.name, &effect) {
            Ok(()) => {}
            Err(error) => {
                return error;
            }
        }
    }

    // All effects have been executed successfully. Depending on the command we either
    // instruct to terminate the sequence or to keep going, enabling the user to rapidly re-
    // trigger the same or some other command.
    if command.is_final {
        CommandExecutionResult::Finished
    } else {
        CommandExecutionResult::KeepGoing
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

fn execute_effect(
    ctx: &Context,
    command_name: &str,
    effect: &Effect,
) -> Result<(), CommandExecutionResult> {
    // Execution happens in a loop to facilitate retry on failure. Every outcome but a retry
    // leaves the loop.
    loop {
        let step = match effect {
            Effect::ShellCommand {
                program,
                is_synchronous: true,
            } => run_to_completion(ctx, command_name, program),
            Effect::ShellCommand {
                program,
                is_synchronous: false,
            } => start_in_background(ctx, program),
        };

        match step {
            Ok(()) => return Ok(()),
            // The user ended the step and already chose how the execution goes on.
            Err(StepFailure::Ended(result)) => return Err(result),
            // A retry would run a second copy beside the command that is still there, so the
            // user may only leave.
            Err(StepFailure::Unkillable(error)) => {
                return Err(match run_unrecoverable_error_mode(ctx, &error) {
                    UnrecoverableErrorResult::Abort => CommandExecutionResult::Finished,
                    UnrecoverableErrorResult::Cancel => CommandExecutionResult::KeepGoing,
                });
            }
            // The command failed cleanly, so the user may also try again.
            Err(StepFailure::Failed(error)) => match run_error_mode(ctx, &error) {
                ErrorResult::Abort => return Err(CommandExecutionResult::Finished),
                ErrorResult::Cancel => return Err(CommandExecutionResult::KeepGoing),
                ErrorResult::Retry => println!("Retrying execution! {error:?}"),
            },
        }
    }
}

/// Why a shell command step did not complete.
enum StepFailure {
    /// The command failed to start or run; it may be retried.
    Failed(ExecutorError),
    /// The command could not be ended and may still be running.
    Unkillable(ExecutorError),
    /// The user ended the step and decided how the execution goes on.
    Ended(CommandExecutionResult),
}

/// Runs the program under the running screen until it finishes or the user kills it.
fn run_to_completion(ctx: &Context, command_name: &str, program: &str) -> Result<(), StepFailure> {
    let running = ctx.executor.start(program).map_err(StepFailure::Failed)?;
    match run_command_running_mode(ctx, command_name, program, running) {
        RunningResult::Finished(result) => result.map(|_output| ()).map_err(StepFailure::Failed),
        RunningResult::KillFailed(error) => Err(StepFailure::Unkillable(error)),
        RunningResult::Cancelled => Err(StepFailure::Ended(CommandExecutionResult::KeepGoing)),
        RunningResult::Aborted => Err(StepFailure::Ended(CommandExecutionResult::Finished)),
    }
}

/// Starts the program and leaves it to run on its own.
fn start_in_background(ctx: &Context, program: &str) -> Result<(), StepFailure> {
    ctx.executor
        .run_in_background(program)
        .map_err(StepFailure::Failed)
}
