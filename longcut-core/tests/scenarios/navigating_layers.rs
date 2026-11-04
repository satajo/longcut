//! Moving through the layer tree: branching, going back, exiting, and running commands that keep
//! the user in the layer.

#![expect(
    clippy::unwrap_used,
    reason = "the helpers build fixtures from literals that are known to be valid"
)]

use crate::harness::Event::KeyPressed;
use crate::harness::{Effect, Scenario, back_key, exit_key};
use crate::helpers::Afterwards::StaysInLayer;
use crate::helpers::Step::Background;
use crate::helpers::{command, key, shows_layers, shows_nothing};
use longcut_core::SessionMode;
use longcut_core::model::layer::Layer;

/// A root layer named root, as the scenario's default one is, with the layer under the key.
fn root_with_layer(shortcut: char, layer: Layer) -> Layer {
    let mut root = Layer::new("root".to_string());
    root.add_layer(key(shortcut), layer).unwrap();
    root
}

/// A root layer named root branching into an empty layer named Tools under t.
fn root_with_empty_tools_under_t() -> Layer {
    root_with_layer('t', Layer::new("Tools".to_string()))
}

#[test]
fn exit_at_the_root_ends_the_session() {
    Scenario::new()
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn shortcut_to_a_layer_branches_into_it_below_the_root() {
    Scenario::new()
        .root_layer(root_with_empty_tools_under_t())
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('t')))
        .expect(shows_layers(&["root", "Tools"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn back_in_a_branch_returns_to_the_parent_and_back_at_the_root_redraws_the_root() {
    Scenario::new()
        .root_layer(root_with_empty_tools_under_t())
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('t')))
        .expect(shows_layers(&["root", "Tools"]))
        .then(KeyPressed(back_key()))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(back_key()))
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn key_bound_to_nothing_redraws_the_same_layer() {
    Scenario::new()
        .root_layer(root_with_empty_tools_under_t())
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('t')))
        .expect(shows_layers(&["root", "Tools"]))
        .then(KeyPressed(key('x')))
        .expect(shows_layers(&["root", "Tools"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}

#[test]
fn non_final_command_runs_and_the_user_stays_in_the_branch() {
    let mut tools = Layer::new("Tools".to_string());
    tools
        .add_command(
            key('n'),
            command("Notify", "notify-send hi", Background, StaysInLayer),
        )
        .unwrap();
    Scenario::new()
        .root_layer(root_with_layer('t', tools))
        .launch(SessionMode::Global)
        .expect(Effect::KeyboardTaken)
        .expect(shows_layers(&["root"]))
        .then(KeyPressed(key('t')))
        .expect(shows_layers(&["root", "Tools"]))
        .then(KeyPressed(key('n')))
        .expect(Effect::Background("notify-send hi".to_string()))
        .expect(shows_layers(&["root", "Tools"]))
        .then(KeyPressed(exit_key()))
        .expect(shows_nothing())
        .expect(Effect::KeyboardReleased)
        .ends();
}
