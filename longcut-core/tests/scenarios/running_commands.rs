//! Synchronous commands: the running screen and its timing, killing a command, and the errors a
//! command can end in, with the retry they offer.

use crate::harness::Event::{
    CommandFinishes, KeyPressed, KillsFail, StartsFail, StartsSucceed, TimePasses,
};
use crate::harness::{Effect, Scenario, Screen, back_key, exit_key};
use crate::helpers::Afterwards::{EndsSession, StaysInLayer};
use crate::helpers::Step::Synchronous;
use crate::helpers::{
    command, command_with_parameters, key, root_with_command, shows_layers, shows_nothing,
    shows_unrecoverable_error,
};
use longcut_core::SessionMode;
use longcut_core::model::command::{Command, CommandParameter};
use longcut_core::model::key::{Key, Symbol};
use longcut_core::model::parameter::{ChooseParameter, ParameterDefinitionVariant};
use longcut_core::port::executor::ExecutorError;

/// The key that confirms a retry. The service fixes it to Return; it is not configured.
fn retry_key() -> Key {
    Key::new(Symbol::RETURN)
}

/// A command named Connect whose one synchronous step connects to a device the user chooses
/// from the options `bluetoothctl devices` generates, and which keeps the user in the layer.
fn connect_to_a_generated_device() -> Command {
    let devices = ChooseParameter::new(None, Some("bluetoothctl devices".to_string()), None)
        .expect("a choose parameter has options or a command generating them");
    let parameter = CommandParameter::new(
        "Device".to_string(),
        ParameterDefinitionVariant::Choose(devices),
    );
    command_with_parameters(
        "Connect",
        "bluetoothctl connect {0}",
        Synchronous,
        StaysInLayer,
        vec![parameter],
    )
}

fn started(program: &str) -> Effect {
    Effect::Started(program.to_string())
}

fn shows_running(command_name: &str, elapsed_secs: u64) -> Effect {
    Effect::Rendered(Screen::Running {
        command_name: command_name.to_string(),
        elapsed_secs,
    })
}

fn shows_error(error_type: &str) -> Effect {
    Effect::Rendered(Screen::Error {
        error_type: error_type.to_string(),
        retry_offered: true,
    })
}

#[test]
fn command_finishing_before_the_delay_shows_no_screen() {
    let volume_up = command("Volume up", "wpctl set-volume", Synchronous, EndsSession);
    Scenario::new()
        .root_layer(root_with_command('c', volume_up))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(started("wpctl set-volume"))
        .then(TimePasses(50))
        .then(CommandFinishes(Ok(String::new())))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn command_outliving_the_delay_shows_the_running_screen_refreshed_each_second() {
    let connect = command("Connect", "bluetoothctl connect", Synchronous, EndsSession);
    Scenario::new()
        .root_layer(root_with_command('c', connect))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(started("bluetoothctl connect"))
        .then(TimePasses(200))
        .expect(shows_running("Connect", 0))
        .then(TimePasses(800))
        .expect(shows_running("Connect", 1))
        .then(TimePasses(1000))
        .expect(shows_running("Connect", 2))
        .then(TimePasses(100))
        .then(CommandFinishes(Ok(String::new())))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn back_key_kills_the_command_and_returns_to_the_layer() {
    let connect = command("Connect", "bluetoothctl connect", Synchronous, EndsSession);
    Scenario::new()
        .root_layer(root_with_command('c', connect))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(started("bluetoothctl connect"))
        .then(TimePasses(200))
        .expect(shows_running("Connect", 0))
        .then(TimePasses(800))
        .expect(shows_running("Connect", 1))
        .then(TimePasses(500))
        .then(KeyPressed(back_key()))
        .expect(Effect::Killed)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn exit_key_kills_the_command_and_ends_the_session() {
    let connect = command("Connect", "bluetoothctl connect", Synchronous, EndsSession);
    Scenario::new()
        .root_layer(root_with_command('c', connect))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(started("bluetoothctl connect"))
        .then(TimePasses(200))
        .expect(shows_running("Connect", 0))
        .then(TimePasses(300))
        .then(KeyPressed(exit_key()))
        .expect(Effect::Killed)
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn other_keys_are_dropped_while_the_command_runs() {
    let connect = command("Connect", "bluetoothctl connect", Synchronous, EndsSession);
    Scenario::new()
        .root_layer(root_with_command('c', connect))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(started("bluetoothctl connect"))
        .then(TimePasses(200))
        .expect(shows_running("Connect", 0))
        .then(TimePasses(100))
        .then(KeyPressed(key('x')))
        .then(TimePasses(300))
        .then(KeyPressed(back_key()))
        .expect(Effect::Killed)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn failed_command_reaches_the_error_screen_with_retry_offered_and_exit_ends_the_session() {
    let connect = command("Connect", "bluetoothctl connect", Synchronous, EndsSession);
    let failure = ExecutorError::Runtime("no such device".to_string());
    Scenario::new()
        .root_layer(root_with_command('c', connect))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(started("bluetoothctl connect"))
        .then(TimePasses(200))
        .expect(shows_running("Connect", 0))
        .then(TimePasses(100))
        .then(CommandFinishes(Err(failure)))
        .expect(shows_error("Runtime error"))
        .then(TimePasses(100))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn failed_command_shows_the_error_with_retry_and_back_returns_to_the_layer() {
    let connect = command("Connect", "bluetoothctl connect", Synchronous, EndsSession);
    let failure = ExecutorError::Runtime("no such device".to_string());
    Scenario::new()
        .root_layer(root_with_command('c', connect))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(started("bluetoothctl connect"))
        .then(TimePasses(100))
        .then(CommandFinishes(Err(failure)))
        .expect(shows_error("Runtime error"))
        .then(KeyPressed(back_key()))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn command_failing_to_start_reaches_the_error_screen() {
    let connect = command("Connect", "bluetoothctl connect", Synchronous, EndsSession);
    Scenario::new()
        .root_layer(root_with_command('c', connect))
        .given(StartsFail)
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(shows_error("Startup error"))
        .then(TimePasses(100))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn retry_starts_the_command_again() {
    let connect = command("Connect", "bluetoothctl connect", Synchronous, EndsSession);
    Scenario::new()
        .root_layer(root_with_command('c', connect))
        .given(StartsFail)
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(shows_error("Startup error"))
        .then(TimePasses(50))
        .then(StartsSucceed)
        .then(TimePasses(50))
        .then(KeyPressed(retry_key()))
        .expect(started("bluetoothctl connect"))
        .then(TimePasses(100))
        .then(CommandFinishes(Ok(String::new())))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn failed_kill_reaches_the_error_screen_without_retry() {
    let connect = command("Connect", "bluetoothctl connect", Synchronous, EndsSession);
    Scenario::new()
        .root_layer(root_with_command('c', connect))
        .given(KillsFail)
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(started("bluetoothctl connect"))
        .then(TimePasses(200))
        .expect(shows_running("Connect", 0))
        .then(TimePasses(300))
        .then(KeyPressed(back_key()))
        .expect(shows_unrecoverable_error("Unknown error"))
        .then(TimePasses(100))
        .then(KeyPressed(back_key()))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn kill_during_option_generation_cancels_the_parameter_input() {
    Scenario::new()
        .root_layer(root_with_command('c', connect_to_a_generated_device()))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(started("bluetoothctl devices"))
        .then(TimePasses(200))
        .expect(shows_running("Device", 0))
        .then(TimePasses(300))
        .then(KeyPressed(back_key()))
        .expect(Effect::Killed)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn failed_kill_during_option_generation_reaches_the_error_screen_without_retry() {
    Scenario::new()
        .root_layer(root_with_command('c', connect_to_a_generated_device()))
        .given(KillsFail)
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('c')))
        .expect(started("bluetoothctl devices"))
        .then(TimePasses(200))
        .expect(shows_running("Device", 0))
        .then(TimePasses(300))
        .then(KeyPressed(back_key()))
        .expect(shows_unrecoverable_error("Unknown error"))
        .then(TimePasses(100))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}
