use std::ops::Deref;

use super::command_execution::{run_command_execution_mode, CommandExecutionResult};
use super::Context;
use crate::model::key::Key;
use crate::model::layer::{Action, Layer};
use crate::port::view::{LayerNavigationViewModel, ViewAction, ViewModel};

/// Enables the user to navigate through the layer tree.
pub fn run_layer_navigation_mode(ctx: &Context) {
    let mut layers = vec![ctx.root_layer];
    loop {
        let active_layer = layers.last().unwrap();
        let is_branched = layers.len() > 1;

        // Rendering
        if is_branched {
            render_branch(ctx, layers.as_slice());
        } else {
            render_root(ctx, active_layer);
        }

        // Input handling
        let press = ctx.input.capture_any();
        if ctx.keys_deactivate.contains(&press) {
            return;
        }

        if is_branched && ctx.keys_back.contains(&press) {
            layers.pop();
            continue;
        }

        if let Some(action) = active_layer.resolve_shortcut(&press) {
            match action {
                Action::Branch(into) => {
                    layers.push(into);
                }
                Action::Execute(command) => {
                    match run_command_execution_mode(ctx, command, &layers) {
                        CommandExecutionResult::KeepGoing => {
                            // Do nothing.
                        }

                        CommandExecutionResult::Finished => {
                            return;
                        }
                    }
                }
            }
        }
    }
}

fn render_root(ctx: &Context, layer: &Layer) {
    let mut actions = render_layer_actions(layer);

    // Deactivate is always available.
    for key in ctx.keys_deactivate {
        actions.push((key, ViewAction::Deactivate));
    }

    render_navigation_view(ctx, actions, &[layer]);
}

fn render_branch(ctx: &Context, layers: &[&Layer]) {
    let mut actions = render_layer_actions(layers.last().unwrap());

    // Back keys are available.
    for key in ctx.keys_back {
        actions.push((key, ViewAction::Unbranch));
    }

    // Deactivate is always available.
    for key in ctx.keys_deactivate {
        actions.push((key, ViewAction::Deactivate));
    }

    render_navigation_view(ctx, actions, layers);
}

fn render_navigation_view(ctx: &Context, actions: Vec<(&Key, ViewAction)>, layers: &[&Layer]) {
    let model = LayerNavigationViewModel {
        actions: &actions,
        layer_stack: layers,
    };

    ctx.view.render(ViewModel::LayerNavigation(model));
}

fn render_layer_actions(layer: &Layer) -> Vec<(&Key, ViewAction)> {
    let mut actions = vec![];

    // Collecting all layer actions into the view action vector.
    for (press, action) in layer.shortcuts.deref() {
        let view_action = match action {
            Action::Branch(layer) => ViewAction::Branch(layer.name.clone()),
            Action::Execute(command) => ViewAction::Execute(command.name.clone()),
        };

        actions.push((press, view_action))
    }

    actions
}
