//! Window launches: the layer of the application whose pattern matches the active window, and
//! the error shown when none does.

use crate::harness::Event::{ActiveWindow, KeyPressed};
use crate::harness::{Effect, Scenario, Screen, back_key, exit_key};
use crate::helpers::{key, shows_layers, shows_nothing};
use longcut_core::SessionMode;
use longcut_core::model::layer::Layer;

fn shows_application_unconfigured() -> Effect {
    Effect::Rendered(Screen::Error {
        error_type: "Application unconfigured".to_string(),
        retry_offered: false,
    })
}

#[test]
fn active_window_matching_a_pattern_navigates_the_first_matching_applications_layer() {
    Scenario::new()
        .app_layer("^Mozilla", Layer::new("Thunderbird".to_string()))
        .app_layer("Firefox$", Layer::new("Firefox".to_string()))
        .given(ActiveWindow("Mozilla Firefox".to_string()))
        .launch(SessionMode::Window)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["Thunderbird"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn active_window_matching_no_pattern_shows_the_unconfigured_error_until_dismissed() {
    Scenario::new()
        .app_layer("Firefox$", Layer::new("Firefox".to_string()))
        .given(ActiveWindow("Alacritty".to_string()))
        .launch(SessionMode::Window)
        .expect(Effect::KeyboardTaken)
        .expect(shows_application_unconfigured())
        .then(KeyPressed(key('x')))
        .then(KeyPressed(back_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn no_active_window_shows_the_unconfigured_error() {
    Scenario::new()
        .app_layer("Firefox$", Layer::new("Firefox".to_string()))
        .launch(SessionMode::Window)
        .expect(Effect::KeyboardTaken)
        .expect(shows_application_unconfigured())
        .then(KeyPressed(back_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}
