use itertools::Itertools;

use super::Context;
use super::command_running::{RunningResult, run_command_running_mode};
use super::error::{UnrecoverableErrorResult, run_unrecoverable_error_mode};
use crate::model::command::{Command, CommandParameter};
use crate::model::key::{Key, Symbol};
use crate::model::layer::Layer;
use crate::model::parameter::{
    CharacterParameter, ChooseParameter, Parameter, ParameterDefinitionVariant,
    ParameterValueVariant, TextParameter,
};
use crate::model::shortcut_map::ShortcutMap;
use crate::port::view;
use crate::port::view::{ParameterInputViewModel, ViewModel};

pub(crate) enum ParameterInputResult {
    Ok(ParameterValueVariant),
    Cancel,
    Exit,
}

pub(crate) struct ParameterInputContext<'a> {
    pub command: &'a Command,
    pub layers: &'a [&'a Layer],
}

/// Processes input from the user to generate values for command parameters.
pub(crate) fn run_parameter_input_mode(
    ctx: &Context,
    context: &ParameterInputContext,
    parameter: &CommandParameter,
) -> ParameterInputResult {
    match &parameter.parameter {
        ParameterDefinitionVariant::Character(definition) => {
            read_character_parameter(ctx, context, &parameter.name, definition)
        }
        ParameterDefinitionVariant::Choose(definition) => {
            read_choose_parameter(ctx, context, &parameter.name, definition)
        }
        ParameterDefinitionVariant::Text(definition) => {
            read_text_parameter(ctx, context, &parameter.name, definition)
        }
    }
}

fn read_character_parameter(
    ctx: &Context,
    context: &ParameterInputContext,
    parameter_name: &str,
    parameter: &CharacterParameter,
) -> ParameterInputResult {
    let view_model = ParameterInputViewModel {
        command: context.command,
        parameter_name,
        parameter: view::ParameterVariant::Character,
        layer_stack: context.layers,
    };
    ctx.view.render(ViewModel::ParameterInput(view_model));

    loop {
        let press = ctx.keyboard.capture_any();
        if ctx.keys_exit.contains(&press) {
            return ParameterInputResult::Exit;
        }

        if ctx.keys_back.contains(&press) {
            return ParameterInputResult::Cancel;
        }

        if let Some(c) = press.symbol.character() {
            if let Ok(value) = parameter.try_assign_value(c) {
                return ParameterInputResult::Ok(ParameterValueVariant::Character(value));
            }
            // Invalid value silently ignored; stop regardless.
            return ParameterInputResult::Exit;
        }
    }
}

fn read_choose_parameter(
    ctx: &Context,
    context: &ParameterInputContext,
    parameter_name: &str,
    parameter: &ChooseParameter,
) -> ParameterInputResult {
    let generated_parameter_options: Vec<String> = match &parameter.gen_options_command {
        None => vec![],
        Some(gen_command) => match generate_options(ctx, parameter_name, gen_command) {
            Ok(output) => output
                .split(&parameter.gen_options_split_by)
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToString::to_string)
                .collect(),
            Err(result) => return result,
        },
    };

    // Pre-configured and generated options are joined up into the same vector of mnemonic
    // pairs. The pre-configured options come first in the vector so they get mapped first,
    // preserving their shortcut mapping regardless of what output the generation command
    // produces.
    let preconfigured_options_iter = parameter.options.iter();
    let generated_options_iter = generated_parameter_options.iter();
    let unique_mnemonics: Vec<(&str, &String)> = preconfigured_options_iter
        .chain(generated_options_iter)
        .unique()
        .map(|option| (option.as_str(), option))
        .collect();

    let mut shortcuts = ShortcutMap::<&String>::new();
    shortcuts.auto_assign_mnemonics(unique_mnemonics);

    // The view is rendered based on the shortcut map content.
    {
        let values: Vec<(&Key, &str)> = shortcuts
            .iter()
            .map(|(key, value)| (key, value.as_str()))
            .collect();

        let view_model = ParameterInputViewModel {
            command: context.command,
            parameter_name,
            parameter: view::ParameterVariant::Choose { options: &values },
            layer_stack: context.layers,
        };
        ctx.view.render(ViewModel::ParameterInput(view_model));
    }

    // With the view render out of the way, we read the input.
    loop {
        let press = ctx.keyboard.capture_any();
        if ctx.keys_exit.contains(&press) {
            return ParameterInputResult::Exit;
        }

        if ctx.keys_back.contains(&press) {
            return ParameterInputResult::Cancel;
        }

        let Some(option) = shortcuts.match_exact(&press) else {
            continue;
        };

        if let Ok(value) = parameter.try_assign_value((*option).clone()) {
            return ParameterInputResult::Ok(ParameterValueVariant::Choose(value));
        }
        // Invalid value silently ignored; stop regardless.
        return ParameterInputResult::Exit;
    }
}

fn read_text_parameter(
    ctx: &Context,
    context: &ParameterInputContext,
    parameter_name: &str,
    parameter: &TextParameter,
) -> ParameterInputResult {
    let mut input = String::new();

    // The view shows the empty input before the first keystroke.
    let view_model = ParameterInputViewModel {
        command: context.command,
        parameter_name,
        parameter: view::ParameterVariant::Text {
            input_value: &input,
        },
        layer_stack: context.layers,
    };
    ctx.view.render(ViewModel::ParameterInput(view_model));

    loop {
        let press = ctx.keyboard.capture_any();
        if ctx.keys_exit.contains(&press) {
            return ParameterInputResult::Exit;
        }

        if ctx.keys_back.contains(&press) && input.is_empty() {
            return ParameterInputResult::Cancel;
        }

        match press.symbol {
            Symbol::RETURN => {
                if let Ok(value) = parameter.try_assign_value(input) {
                    return ParameterInputResult::Ok(ParameterValueVariant::Text(value));
                }
                // Invalid value. Silently ignored for now.
                input = String::new();
            }
            Symbol::BACKSPACE => {
                input.pop();
            }
            symbol => {
                if let Some(c) = symbol.character() {
                    input.push(c);
                }
                // Keys typing nothing are irrelevant input.
            }
        }

        // Re-render after each keystroke.
        let view_model = ParameterInputViewModel {
            command: context.command,
            parameter_name,
            parameter: view::ParameterVariant::Text {
                input_value: &input,
            },
            layer_stack: context.layers,
        };
        ctx.view.render(ViewModel::ParameterInput(view_model));
    }
}

/// Runs the option generation command under the running screen, titled with the parameter. A
/// command that fails to start or fails generates no options; the preconfigured options stand.
/// A command the user ended, or asked to end but could not be ended, ends the input instead.
fn generate_options(
    ctx: &Context,
    parameter_name: &str,
    gen_command: &str,
) -> Result<String, ParameterInputResult> {
    let Ok(running) = ctx.executor.start(gen_command) else {
        return Ok(String::new());
    };
    match run_command_running_mode(ctx, parameter_name, gen_command, running) {
        RunningResult::Finished(Ok(output)) => Ok(output),
        RunningResult::Finished(Err(_)) => Ok(String::new()),
        RunningResult::Cancelled => Err(ParameterInputResult::Cancel),
        RunningResult::Aborted => Err(ParameterInputResult::Exit),
        RunningResult::KillFailed(error) => Err(match run_unrecoverable_error_mode(ctx, &error) {
            UnrecoverableErrorResult::Abort => ParameterInputResult::Exit,
            UnrecoverableErrorResult::Cancel => ParameterInputResult::Cancel,
        }),
    }
}
