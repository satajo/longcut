use super::Context;
use super::command_execution::{CommandExecutionResult, run_command_execution_mode};
use crate::model::key::Key;
use crate::model::layer::{Action, Layer};
use crate::port::view::{LayerNavigationViewModel, ViewAction, ViewModel};
use std::iter;

/// Enables the user to navigate through the layer tree.
pub(crate) fn run_layer_navigation_mode(ctx: &Context) {
    // The layers branched into below the root, innermost last.
    let mut branches: Vec<&Layer> = Vec::new();
    loop {
        let active_layer = branches.last().copied().unwrap_or(ctx.root_layer);
        let layers: Vec<&Layer> = iter::once(ctx.root_layer)
            .chain(branches.iter().copied())
            .collect();

        // Rendering
        if branches.is_empty() {
            render_root(ctx, active_layer);
        } else {
            render_branch(ctx, active_layer, &layers);
        }

        // Input handling
        let press = ctx.keyboard.capture_any();
        if ctx.keys_exit.contains(&press) {
            return;
        }

        if !branches.is_empty() && ctx.keys_back.contains(&press) {
            branches.pop();
            continue;
        }

        if let Some(action) = active_layer.resolve_shortcut(&press) {
            match action {
                Action::Branch(into) => {
                    branches.push(into);
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

    // Exit is always available.
    for key in ctx.keys_exit {
        actions.push((key, ViewAction::Exit));
    }

    render_navigation_view(ctx, &actions, &[layer]);
}

fn render_branch(ctx: &Context, active_layer: &Layer, layers: &[&Layer]) {
    let mut actions = render_layer_actions(active_layer);

    // Back keys are available.
    for key in ctx.keys_back {
        actions.push((key, ViewAction::Unbranch));
    }

    // Exit is always available.
    for key in ctx.keys_exit {
        actions.push((key, ViewAction::Exit));
    }

    render_navigation_view(ctx, &actions, layers);
}

fn render_navigation_view(ctx: &Context, actions: &[(&Key, ViewAction)], layers: &[&Layer]) {
    let model = LayerNavigationViewModel {
        actions,
        layer_stack: layers,
    };

    ctx.view.render(ViewModel::LayerNavigation(model));
}

fn render_layer_actions(layer: &Layer) -> Vec<(&Key, ViewAction)> {
    let mut actions = vec![];

    // Collecting all layer actions into the view action vector.
    for (press, action) in &*layer.shortcuts {
        let view_action = match action {
            Action::Branch(layer) => ViewAction::Branch(layer.name.clone()),
            Action::Execute(command) => ViewAction::Execute(command.name.clone()),
        };

        actions.push((press, view_action));
    }

    actions
}
