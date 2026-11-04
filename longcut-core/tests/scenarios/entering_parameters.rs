//! Parameter input: typing and editing text, single characters, and choosing among preconfigured
//! and generated options by their mnemonics.

#![expect(
    clippy::unwrap_used,
    reason = "the helpers build fixtures from literals that are known to be valid"
)]

use crate::harness::Event::{CommandFinishes, KeyPressed, TimePasses};
use crate::harness::{Effect, ParameterScreen, Scenario, Screen, back_key, exit_key};
use crate::helpers::Afterwards::StaysInLayer;
use crate::helpers::Step::Background;
use crate::helpers::{
    command_with_parameters, key, root_with_command, shows_layers, shows_nothing,
};
use longcut_core::SessionMode;
use longcut_core::model::command::CommandParameter;
use longcut_core::model::key::{Key, Symbol};
use longcut_core::model::layer::Layer;
use longcut_core::model::parameter::{
    CharacterParameter, ChooseParameter, ParameterDefinitionVariant, TextParameter,
};
use longcut_core::port::executor::ExecutorError;
use std::time::Duration;

fn return_key() -> Key {
    Key::new(Symbol::RETURN)
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
        ChooseParameter::new(Some(options), gen_command.map(ToString::to_string), None).unwrap();
    CommandParameter::new(
        name.to_string(),
        ParameterDefinitionVariant::Choose(definition),
    )
}

/// A root layer named root with a command named Echo under e, which runs the program in the
/// background with the parameters substituted into it and keeps the user in the layer.
fn root_with_echo_under_e(program: &str, parameters: Vec<CommandParameter>) -> Layer {
    root_with_command(
        'e',
        command_with_parameters("Echo", program, Background, StaysInLayer, parameters),
    )
}

fn shows_parameter_input(parameter_name: &str, input: ParameterScreen) -> Effect {
    Effect::Rendered(Screen::ParameterInput {
        parameter_name: parameter_name.to_string(),
        input,
    })
}

fn shows_character_input(parameter_name: &str) -> Effect {
    shows_parameter_input(parameter_name, ParameterScreen::Character)
}

/// The text input screen with the text typed so far.
fn shows_text_input(parameter_name: &str, typed: &str) -> Effect {
    shows_parameter_input(parameter_name, ParameterScreen::Text(typed.to_string()))
}

fn shows_options(parameter_name: &str, options: &[(&str, &str)]) -> Effect {
    shows_parameter_input(
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
        .root_layer(root_with_echo_under_e("echo {0}", vec![text("Message")]))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('e')))
        .expect(shows_text_input("Message", ""))
        .then(KeyPressed(key('h')))
        .expect(shows_text_input("Message", "h"))
        .then(KeyPressed(key('i')))
        .expect(shows_text_input("Message", "hi"))
        .then(KeyPressed(return_key()))
        .expect(background("echo hi"))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn back_deletes_the_last_typed_character_and_on_empty_text_cancels_to_the_layer() {
    Scenario::new()
        .root_layer(root_with_echo_under_e("echo {0}", vec![text("Message")]))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('e')))
        .expect(shows_text_input("Message", ""))
        .then(KeyPressed(key('h')))
        .expect(shows_text_input("Message", "h"))
        .then(KeyPressed(key('i')))
        .expect(shows_text_input("Message", "hi"))
        .then(KeyPressed(back_key()))
        .expect(shows_text_input("Message", "h"))
        .then(KeyPressed(back_key()))
        .expect(shows_text_input("Message", ""))
        .then(KeyPressed(back_key()))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn choose_shows_the_options_under_their_mnemonics_and_a_mnemonic_runs_with_its_option() {
    Scenario::new()
        .root_layer(root_with_echo_under_e(
            "echo {0}",
            vec![choose("Greek", &["alpha", "beta"], None)],
        ))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('e')))
        .expect(shows_options("Greek", &[("a", "alpha"), ("b", "beta")]))
        .then(KeyPressed(key('b')))
        .expect(background("echo beta"))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn generated_options_join_after_the_preconfigured_ones_without_duplicates() {
    Scenario::new()
        .root_layer(root_with_echo_under_e(
            "echo {0}",
            vec![choose("Fruit", &["alpha", "beta"], Some("list-fruit"))],
        ))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('e')))
        .expect(Effect::Started("list-fruit".to_string()))
        .then(TimePasses(Duration::from_millis(100)))
        .then(CommandFinishes(Ok("beta\n apple \n\n".to_string())))
        .expect(shows_options(
            "Fruit",
            &[("a", "alpha"), ("b", "beta"), ("p", "apple")],
        ))
        .then(KeyPressed(key('p')))
        .expect(background("echo apple"))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn generated_options_split_on_the_configured_separator() {
    let definition =
        ChooseParameter::new(None, Some("list-fruit".to_string()), Some(",".to_string())).unwrap();
    let parameter = CommandParameter::new(
        "Fruit".to_string(),
        ParameterDefinitionVariant::Choose(definition),
    );
    Scenario::new()
        .root_layer(root_with_echo_under_e("echo {0}", vec![parameter]))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('e')))
        .expect(Effect::Started("list-fruit".to_string()))
        .then(TimePasses(Duration::from_millis(100)))
        .then(CommandFinishes(Ok("apple, cherry\n".to_string())))
        .expect(shows_options("Fruit", &[("a", "apple"), ("c", "cherry")]))
        .then(KeyPressed(key('c')))
        .expect(background("echo cherry"))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn failed_option_generation_leaves_only_the_preconfigured_options() {
    let failure = ExecutorError::Runtime("no such file".to_string());
    Scenario::new()
        .root_layer(root_with_echo_under_e(
            "echo {0}",
            vec![choose("Fruit", &["alpha", "beta"], Some("list-fruit"))],
        ))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('e')))
        .expect(Effect::Started("list-fruit".to_string()))
        .then(TimePasses(Duration::from_millis(100)))
        .then(CommandFinishes(Err(failure)))
        .expect(shows_options("Fruit", &[("a", "alpha"), ("b", "beta")]))
        .then(KeyPressed(key('a')))
        .expect(background("echo alpha"))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn key_that_is_no_mnemonic_is_ignored_without_redrawing_the_options() {
    Scenario::new()
        .root_layer(root_with_echo_under_e(
            "echo {0}",
            vec![choose("Greek", &["alpha", "beta"], None)],
        ))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('e')))
        .expect(shows_options("Greek", &[("a", "alpha"), ("b", "beta")]))
        .then(KeyPressed(key('x')))
        .then(KeyPressed(key('a')))
        .expect(background("echo alpha"))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn cancelling_a_later_parameter_returns_to_the_layer_without_running_the_command() {
    Scenario::new()
        .root_layer(root_with_echo_under_e(
            "echo {0} {1}",
            vec![text("Greeting"), text("Name")],
        ))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('e')))
        .expect(shows_text_input("Greeting", ""))
        .then(KeyPressed(key('h')))
        .expect(shows_text_input("Greeting", "h"))
        .then(KeyPressed(return_key()))
        .expect(shows_text_input("Name", ""))
        .then(KeyPressed(back_key()))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn exit_during_parameter_input_ends_the_session_without_running_the_command() {
    Scenario::new()
        .root_layer(root_with_echo_under_e("echo {0}", vec![text("Message")]))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('e')))
        .expect(shows_text_input("Message", ""))
        .then(KeyPressed(key('h')))
        .expect(shows_text_input("Message", "h"))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn character_parameter_runs_the_command_with_the_first_typed_character() {
    Scenario::new()
        .root_layer(root_with_echo_under_e(
            "echo {0}",
            vec![character("Letter")],
        ))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('e')))
        .expect(shows_character_input("Letter"))
        .then(KeyPressed(key('x')))
        .expect(background("echo x"))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn key_typing_no_character_is_ignored_by_a_character_parameter() {
    let f1 = Key::new("F1".parse().unwrap());
    Scenario::new()
        .root_layer(root_with_echo_under_e(
            "echo {0}",
            vec![character("Letter")],
        ))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('e')))
        .expect(shows_character_input("Letter"))
        .then(KeyPressed(f1))
        .then(KeyPressed(key('x')))
        .expect(background("echo x"))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}
