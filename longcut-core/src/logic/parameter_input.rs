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

#[cfg(test)]
mod tests {
    use crate::logic::run_layer_navigation_mode;
    use crate::model::command::{Command, CommandParameter};
    use crate::model::effect::{EffectTemplate, ShellCommandTemplate};
    use crate::model::key::{Key, Symbol};
    use crate::model::layer::Layer;
    use crate::model::parameter::{
        CharacterParameter, ChooseParameter, ParameterDefinitionVariant, TextParameter,
    };
    use crate::port::executor::ExecutorError;
    use crate::port::fakes::Event::{CommandFinishes, KeyPressed};
    use crate::port::fakes::{Effect, ParameterScreen, Scenario, Screen};
    use xkbcommon::xkb;

    fn key(c: char) -> Key {
        Key::new(Symbol::from_char(c).unwrap())
    }

    fn return_key() -> Key {
        Key::new(Symbol::RETURN)
    }

    fn back_key() -> Key {
        Key::new(Symbol::BACKSPACE)
    }

    fn exit_key() -> Key {
        Key::new(Symbol::ESCAPE)
    }

    fn text(name: &str) -> CommandParameter {
        CommandParameter::new(
            name.to_string(),
            ParameterDefinitionVariant::Text(TextParameter),
        )
    }

    fn character(name: &str) -> CommandParameter {
        CommandParameter::new(
            name.to_string(),
            ParameterDefinitionVariant::Character(CharacterParameter),
        )
    }

    fn choose(name: &str, options: &[&str], gen_command: Option<&str>) -> CommandParameter {
        let options = options.iter().map(ToString::to_string).collect();
        let definition =
            ChooseParameter::new(Some(options), gen_command.map(ToString::to_string), None)
                .unwrap();
        CommandParameter::new(
            name.to_string(),
            ParameterDefinitionVariant::Choose(definition),
        )
    }

    /// A root layer with a non-final command under e running the program in the background,
    /// with the parameters substituted into it.
    fn root_with_command(program: &str, parameters: Vec<CommandParameter>) -> Layer {
        let step = ShellCommandTemplate::new(program).unwrap();
        let steps = vec![EffectTemplate::ShellCommand(step)];
        let command = Command::new("Echo".to_string(), steps, parameters).unwrap();
        let mut root = Layer::new("root".to_string());
        root.add_command(key('e'), command).unwrap();
        root
    }

    fn root_screen() -> Effect {
        Effect::Rendered(Screen::LayerNavigation {
            layer_stack: vec!["root".to_string()],
        })
    }

    fn parameter_screen(parameter_name: &str, input: ParameterScreen) -> Effect {
        Effect::Rendered(Screen::ParameterInput {
            parameter_name: parameter_name.to_string(),
            input,
        })
    }

    fn text_screen(parameter_name: &str, input: &str) -> Effect {
        parameter_screen(parameter_name, ParameterScreen::Text(input.to_string()))
    }

    fn choose_screen(parameter_name: &str, options: &[(&str, &str)]) -> Effect {
        parameter_screen(
            parameter_name,
            ParameterScreen::Choose(
                options
                    .iter()
                    .map(|(key, option)| ((*key).to_string(), (*option).to_string()))
                    .collect(),
            ),
        )
    }

    fn background(program: &str) -> Effect {
        Effect::Background(program.to_string())
    }

    #[test]
    fn typed_text_shows_after_each_key_and_return_runs_the_command_with_it() {
        Scenario::new()
            .root_layer(root_with_command("echo {0}", vec![text("Message")]))
            .run(run_layer_navigation_mode)
            .effect(root_screen())
            .then(KeyPressed(key('e')))
            .effect(text_screen("Message", ""))
            .then(KeyPressed(key('h')))
            .effect(text_screen("Message", "h"))
            .then(KeyPressed(key('i')))
            .effect(text_screen("Message", "hi"))
            .then(KeyPressed(return_key()))
            .effect(background("echo hi"))
            .effect(root_screen())
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn back_deletes_the_last_typed_character_and_on_empty_text_cancels_to_the_layer() {
        Scenario::new()
            .root_layer(root_with_command("echo {0}", vec![text("Message")]))
            .run(run_layer_navigation_mode)
            .effect(root_screen())
            .then(KeyPressed(key('e')))
            .effect(text_screen("Message", ""))
            .then(KeyPressed(key('h')))
            .effect(text_screen("Message", "h"))
            .then(KeyPressed(key('i')))
            .effect(text_screen("Message", "hi"))
            .then(KeyPressed(back_key()))
            .effect(text_screen("Message", "h"))
            .then(KeyPressed(back_key()))
            .effect(text_screen("Message", ""))
            .then(KeyPressed(back_key()))
            .effect(root_screen())
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn choose_shows_the_options_under_their_mnemonics_and_a_mnemonic_runs_with_its_option() {
        Scenario::new()
            .root_layer(root_with_command(
                "echo {0}",
                vec![choose("Greek", &["alpha", "beta"], None)],
            ))
            .run(run_layer_navigation_mode)
            .effect(root_screen())
            .then(KeyPressed(key('e')))
            .effect(choose_screen("Greek", &[("a", "alpha"), ("b", "beta")]))
            .then(KeyPressed(key('b')))
            .effect(background("echo beta"))
            .effect(root_screen())
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn generated_options_join_after_the_preconfigured_ones_without_duplicates() {
        Scenario::new()
            .root_layer(root_with_command(
                "echo {0}",
                vec![choose("Fruit", &["alpha", "beta"], Some("list-fruit"))],
            ))
            .run(run_layer_navigation_mode)
            .effect(root_screen())
            .then(KeyPressed(key('e')))
            .effect(Effect::Started("list-fruit".to_string()))
            .at(100, CommandFinishes(Ok("beta\n apple \n\n".to_string())))
            .effect(choose_screen(
                "Fruit",
                &[("a", "alpha"), ("b", "beta"), ("p", "apple")],
            ))
            .then(KeyPressed(key('p')))
            .effect(background("echo apple"))
            .effect(root_screen())
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn generated_options_split_on_the_configured_separator() {
        let definition =
            ChooseParameter::new(None, Some("list-fruit".to_string()), Some(",".to_string()))
                .unwrap();
        let parameter = CommandParameter::new(
            "Fruit".to_string(),
            ParameterDefinitionVariant::Choose(definition),
        );
        Scenario::new()
            .root_layer(root_with_command("echo {0}", vec![parameter]))
            .run(run_layer_navigation_mode)
            .effect(root_screen())
            .then(KeyPressed(key('e')))
            .effect(Effect::Started("list-fruit".to_string()))
            .at(100, CommandFinishes(Ok("apple, cherry\n".to_string())))
            .effect(choose_screen("Fruit", &[("a", "apple"), ("c", "cherry")]))
            .then(KeyPressed(key('c')))
            .effect(background("echo cherry"))
            .effect(root_screen())
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn failed_option_generation_leaves_only_the_preconfigured_options() {
        let failure = ExecutorError::Runtime("no such file".to_string());
        Scenario::new()
            .root_layer(root_with_command(
                "echo {0}",
                vec![choose("Fruit", &["alpha", "beta"], Some("list-fruit"))],
            ))
            .run(run_layer_navigation_mode)
            .effect(root_screen())
            .then(KeyPressed(key('e')))
            .effect(Effect::Started("list-fruit".to_string()))
            .at(100, CommandFinishes(Err(failure)))
            .effect(choose_screen("Fruit", &[("a", "alpha"), ("b", "beta")]))
            .then(KeyPressed(key('a')))
            .effect(background("echo alpha"))
            .effect(root_screen())
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn key_that_is_no_mnemonic_is_ignored_without_redrawing_the_options() {
        Scenario::new()
            .root_layer(root_with_command(
                "echo {0}",
                vec![choose("Greek", &["alpha", "beta"], None)],
            ))
            .run(run_layer_navigation_mode)
            .effect(root_screen())
            .then(KeyPressed(key('e')))
            .effect(choose_screen("Greek", &[("a", "alpha"), ("b", "beta")]))
            .then(KeyPressed(key('x')))
            .then(KeyPressed(key('a')))
            .effect(background("echo alpha"))
            .effect(root_screen())
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn cancelling_a_later_parameter_returns_to_the_layer_without_running_the_command() {
        Scenario::new()
            .root_layer(root_with_command(
                "echo {0} {1}",
                vec![text("Greeting"), text("Name")],
            ))
            .run(run_layer_navigation_mode)
            .effect(root_screen())
            .then(KeyPressed(key('e')))
            .effect(text_screen("Greeting", ""))
            .then(KeyPressed(key('h')))
            .effect(text_screen("Greeting", "h"))
            .then(KeyPressed(return_key()))
            .effect(text_screen("Name", ""))
            .then(KeyPressed(back_key()))
            .effect(root_screen())
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn exit_during_parameter_input_ends_the_session_without_running_the_command() {
        Scenario::new()
            .root_layer(root_with_command("echo {0}", vec![text("Message")]))
            .run(run_layer_navigation_mode)
            .effect(root_screen())
            .then(KeyPressed(key('e')))
            .effect(text_screen("Message", ""))
            .then(KeyPressed(key('h')))
            .effect(text_screen("Message", "h"))
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn character_parameter_runs_the_command_with_the_first_typed_character() {
        Scenario::new()
            .root_layer(root_with_command("echo {0}", vec![character("Letter")]))
            .run(run_layer_navigation_mode)
            .effect(root_screen())
            .then(KeyPressed(key('e')))
            .effect(parameter_screen("Letter", ParameterScreen::Character))
            .then(KeyPressed(key('x')))
            .effect(background("echo x"))
            .effect(root_screen())
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn key_typing_no_character_is_ignored_by_a_character_parameter() {
        let f1 = Key::new(Symbol::from_keysym(xkb::Keysym::F1).unwrap());
        Scenario::new()
            .root_layer(root_with_command("echo {0}", vec![character("Letter")]))
            .run(run_layer_navigation_mode)
            .effect(root_screen())
            .then(KeyPressed(key('e')))
            .effect(parameter_screen("Letter", ParameterScreen::Character))
            .then(KeyPressed(f1))
            .then(KeyPressed(key('x')))
            .effect(background("echo x"))
            .effect(root_screen())
            .then(KeyPressed(exit_key()))
            .ends();
    }
}
