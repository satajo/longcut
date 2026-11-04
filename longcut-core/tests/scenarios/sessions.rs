//! A launch as a whole: which root layer the session mode picks, how a session holds the keyboard
//! and leaves the screen, and what a launch shows when the keyboard cannot be taken.

use crate::harness::Event::{ActiveWindow, KeyPressed, KeyboardUnavailable, TimePasses};
use crate::harness::{Effect, Scenario, exit_key};
use crate::helpers::Afterwards::EndsSession;
use crate::helpers::Step::Background;
use crate::helpers::{command, key, shows_layers, shows_nothing, shows_unrecoverable_error};
use longcut_core::SessionMode;
use longcut_core::model::layer::Layer;

#[test]
fn global_launch_navigates_the_root_layer_on_the_taken_keyboard() {
    Scenario::new()
        .root_layer(Layer::new("Global".to_string()))
        .app_layer("Firefox$", Layer::new("Firefox".to_string()))
        .given(ActiveWindow("Mozilla Firefox".to_string()))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["Global"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn window_launch_navigates_the_active_windows_layer_on_the_taken_keyboard() {
    Scenario::new()
        .root_layer(Layer::new("Global".to_string()))
        .app_layer("Firefox$", Layer::new("Firefox".to_string()))
        .given(ActiveWindow("Mozilla Firefox".to_string()))
        .launch(SessionMode::Window)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["Firefox"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn final_command_ends_the_session_and_clears_the_screen_before_giving_the_keyboard_back() {
    let mut root = Layer::new("Global".to_string());
    root.add_command(
        key('f'),
        command("Firefox", "firefox", Background, EndsSession),
    )
    .unwrap();

    Scenario::new()
        .root_layer(root)
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["Global"]))
        .then(KeyPressed(key('f')))
        .expect(Effect::Background("firefox".to_string()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn launch_without_the_keyboard_shows_why_for_four_seconds_then_clears() {
    Scenario::new()
        .given(KeyboardUnavailable)
        .launch(SessionMode::Global)
        .expect(shows_unrecoverable_error("Keyboard unavailable"))
        .then(TimePasses(4000))
        .expect(shows_nothing())
        .ends();
}
