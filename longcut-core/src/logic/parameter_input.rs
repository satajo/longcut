use itertools::Itertools;

use super::Context;
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

pub enum ParameterInputResult {
    Ok(ParameterValueVariant),
    Cancel,
    Exit,
}

pub struct ParameterInputContext<'a> {
    pub command: &'a Command,
    pub layers: &'a [&'a Layer],
}

/// Processes input from the user to generate values for command parameters.
pub fn run_parameter_input_mode(
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
        parameter: view::ParameterVariant::CharInput,
        layer_stack: context.layers,
    };
    ctx.view.render(ViewModel::ParameterInput(view_model));

    loop {
        let press = ctx.input.capture_any();

        if ctx.keys_deactivate.contains(&press) {
            return ParameterInputResult::Exit;
        }

        if ctx.keys_back.contains(&press) {
            return ParameterInputResult::Cancel;
        }

        match press.symbol {
            Symbol::Character(c) => {
                let Ok(value) = parameter.try_assign_value(c) else {
                    // Invalid value. Silently ignored for now, but could be handled using the error screen?
                    continue;
                };

                return ParameterInputResult::Ok(ParameterValueVariant::Character(value));
            }
            _ => { /* Irrelevant input. */ }
        }
    }
}

fn read_choose_parameter(
    ctx: &Context,
    context: &ParameterInputContext,
    parameter_name: &str,
    parameter: &ChooseParameter,
) -> ParameterInputResult {
    let generated_parameter_options: Vec<String> =
        if let Some(gen_command) = &parameter.gen_options_command {
            if let Ok(output) = ctx.executor.run_to_completion(gen_command) {
                output
                    .split(&parameter.gen_options_split_by)
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .map(|line| line.to_string())
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
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
            parameter: view::ParameterVariant::OptionInput { options: &values },
            layer_stack: context.layers,
        };
        ctx.view.render(ViewModel::ParameterInput(view_model));
    }

    // With the view render out of the way, we read the input.
    loop {
        let press = ctx.input.capture_any();

        if ctx.keys_deactivate.contains(&press) {
            return ParameterInputResult::Exit;
        }

        if ctx.keys_back.contains(&press) {
            return ParameterInputResult::Cancel;
        }

        let Some(option) = shortcuts.match_fuzzy(&press) else {
            continue;
        };

        let Ok(value) = parameter.try_assign_value(option.to_string()) else {
            // Invalid value. Silently ignored for now, but could be handled using the error screen?
            continue;
        };

        return ParameterInputResult::Ok(ParameterValueVariant::Choose(value));
    }
}

fn read_text_parameter(
    ctx: &Context,
    context: &ParameterInputContext,
    parameter_name: &str,
    parameter: &TextParameter,
) -> ParameterInputResult {
    let mut input = String::new();
    loop {
        let view_model = ParameterInputViewModel {
            command: context.command,
            parameter_name,
            parameter: view::ParameterVariant::StringInput {
                input_value: &input,
            },
            layer_stack: context.layers,
        };
        ctx.view.render(ViewModel::ParameterInput(view_model));

        let press = ctx.input.capture_any();

        if ctx.keys_deactivate.contains(&press) {
            return ParameterInputResult::Exit;
        }

        if ctx.keys_back.contains(&press) && input.is_empty() {
            return ParameterInputResult::Cancel;
        }

        match press.symbol {
            Symbol::Character(c) => input.push(c),
            Symbol::Return => {
                let Ok(value) = parameter.try_assign_value(input) else {
                    // Invalid value. Silently ignored for now, but could be handled using the error screen?
                    input = String::new();
                    continue;
                };

                return ParameterInputResult::Ok(ParameterValueVariant::Text(value));
            }
            Symbol::BackSpace => {
                input.pop();
            }
            _ => { /* Irrelevant input. */ }
        }
    }
}
