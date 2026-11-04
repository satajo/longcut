//! Building blocks the scenarios of several behaviour areas share.

use crate::harness::{Effect, Screen};
use longcut_core::model::command::{Command, CommandParameter};
use longcut_core::model::effect::{EffectTemplate, ShellCommandTemplate};
use longcut_core::model::key::{Key, Symbol};
use longcut_core::model::layer::Layer;

/// The key that types the character.
pub(crate) fn key(c: char) -> Key {
    Key::new(Symbol::from_char(c).expect("the character spells a keysym"))
}

/// How the one step of a command runs.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Step {
    Background,
    Synchronous,
}

/// Where the user is once a command has run.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Afterwards {
    StaysInLayer,
    EndsSession,
}

/// A command with one step running the program and no parameters.
pub(crate) fn command(name: &str, program: &str, step: Step, afterwards: Afterwards) -> Command {
    command_with_parameters(name, program, step, afterwards, vec![])
}

/// A command with one step running the program, with the parameters substituted into it.
pub(crate) fn command_with_parameters(
    name: &str,
    program: &str,
    step: Step,
    afterwards: Afterwards,
    parameters: Vec<CommandParameter>,
) -> Command {
    let mut template =
        ShellCommandTemplate::new(program).expect("the program is a shell command template");
    template.set_synchronous(matches!(step, Step::Synchronous));
    let steps = vec![EffectTemplate::ShellCommand(template)];
    let mut command = Command::new(name.to_string(), steps, parameters)
        .expect("the program substitutes every parameter and no other");
    command.set_final(matches!(afterwards, Afterwards::EndsSession));
    command
}

/// A root layer named root, as the scenario's default one is, with the command under the key.
pub(crate) fn root_with_command(shortcut: char, command: Command) -> Layer {
    let mut root = Layer::new("root".to_string());
    root.add_command(key(shortcut), command)
        .expect("the key is unbound in the root layer");
    root
}

/// The layer navigation screen with the named layers navigated into, root first.
pub(crate) fn shows_layers(layer_stack: &[&str]) -> Effect {
    Effect::Rendered(Screen::LayerNavigation {
        layer_stack: layer_stack.iter().map(ToString::to_string).collect(),
    })
}

/// The screen cleared, as it is when a session ends.
pub(crate) fn shows_nothing() -> Effect {
    Effect::Rendered(Screen::None)
}

/// The error screen of the error type, offering no retry.
pub(crate) fn shows_unrecoverable_error(error_type: &str) -> Effect {
    Effect::Rendered(Screen::Error {
        error_type: error_type.to_string(),
        retry_offered: false,
    })
}
