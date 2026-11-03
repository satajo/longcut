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

#[cfg(test)]
mod tests {
    use super::run_layer_navigation_mode;
    use crate::model::command::Command;
    use crate::model::effect::{EffectTemplate, ShellCommandTemplate};
    use crate::model::key::{Key, Symbol};
    use crate::model::layer::Layer;
    use crate::port::executor::ExecutorError;
    use crate::port::fakes::Event::{CommandFinishes, KeyPressed};
    use crate::port::fakes::{Effect, Scenario, Screen};

    fn key(c: char) -> Key {
        Key::new(Symbol::from_char(c).unwrap())
    }

    fn back_key() -> Key {
        Key::new(Symbol::BACKSPACE)
    }

    fn exit_key() -> Key {
        Key::new(Symbol::ESCAPE)
    }

    fn command(name: &str, program: &str, synchronous: bool, is_final: bool) -> Command {
        let mut step = ShellCommandTemplate::new(program).unwrap();
        step.set_synchronous(synchronous);
        let steps = vec![EffectTemplate::ShellCommand(step)];
        let mut command = Command::new(name.to_string(), steps, vec![]).unwrap();
        command.set_final(is_final);
        command
    }

    /// A root layer with the layer under the key.
    fn root_with_layer(shortcut: char, layer: Layer) -> Layer {
        let mut root = Layer::new("root".to_string());
        root.add_layer(key(shortcut), layer).unwrap();
        root
    }

    /// A root layer with the command under the key.
    fn root_with_command(shortcut: char, command: Command) -> Layer {
        let mut root = Layer::new("root".to_string());
        root.add_command(key(shortcut), command).unwrap();
        root
    }

    /// A root layer branching into a layer named Tools under t.
    fn root_with_tools() -> Layer {
        root_with_layer('t', Layer::new("Tools".to_string()))
    }

    fn navigation(layer_stack: &[&str]) -> Effect {
        Effect::Rendered(Screen::LayerNavigation {
            layer_stack: layer_stack.iter().map(ToString::to_string).collect(),
        })
    }

    fn error(error_type: &str, retry_offered: bool) -> Effect {
        Effect::Rendered(Screen::Error {
            error_type: error_type.to_string(),
            retry_offered,
        })
    }

    #[test]
    fn exit_at_the_root_ends_the_session() {
        Scenario::new()
            .run(run_layer_navigation_mode)
            .effect(navigation(&["root"]))
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn shortcut_to_a_layer_branches_into_it_below_the_root() {
        Scenario::new()
            .root_layer(root_with_tools())
            .run(run_layer_navigation_mode)
            .effect(navigation(&["root"]))
            .then(KeyPressed(key('t')))
            .effect(navigation(&["root", "Tools"]))
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn back_in_a_branch_returns_to_the_parent_and_back_at_the_root_redraws_the_root() {
        Scenario::new()
            .root_layer(root_with_tools())
            .run(run_layer_navigation_mode)
            .effect(navigation(&["root"]))
            .then(KeyPressed(key('t')))
            .effect(navigation(&["root", "Tools"]))
            .then(KeyPressed(back_key()))
            .effect(navigation(&["root"]))
            .then(KeyPressed(back_key()))
            .effect(navigation(&["root"]))
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn key_bound_to_nothing_redraws_the_same_layer() {
        Scenario::new()
            .root_layer(root_with_tools())
            .run(run_layer_navigation_mode)
            .effect(navigation(&["root"]))
            .then(KeyPressed(key('t')))
            .effect(navigation(&["root", "Tools"]))
            .then(KeyPressed(key('x')))
            .effect(navigation(&["root", "Tools"]))
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn non_final_command_runs_and_the_user_stays_in_the_branch() {
        let mut tools = Layer::new("Tools".to_string());
        tools
            .add_command(key('n'), command("Notify", "notify-send hi", false, false))
            .unwrap();
        Scenario::new()
            .root_layer(root_with_layer('t', tools))
            .run(run_layer_navigation_mode)
            .effect(navigation(&["root"]))
            .then(KeyPressed(key('t')))
            .effect(navigation(&["root", "Tools"]))
            .then(KeyPressed(key('n')))
            .effect(Effect::Background("notify-send hi".to_string()))
            .effect(navigation(&["root", "Tools"]))
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn final_command_ends_the_session_once_it_has_run() {
        Scenario::new()
            .root_layer(root_with_command(
                'f',
                command("Firefox", "firefox", false, true),
            ))
            .run(run_layer_navigation_mode)
            .effect(navigation(&["root"]))
            .then(KeyPressed(key('f')))
            .effect(Effect::Background("firefox".to_string()))
            .ends();
    }

    #[test]
    fn exit_in_a_branch_ends_the_session() {
        Scenario::new()
            .root_layer(root_with_tools())
            .run(run_layer_navigation_mode)
            .effect(navigation(&["root"]))
            .then(KeyPressed(key('t')))
            .effect(navigation(&["root", "Tools"]))
            .then(KeyPressed(exit_key()))
            .ends();
    }

    #[test]
    fn failed_command_shows_the_error_with_retry_and_back_returns_to_the_layer() {
        let failure = ExecutorError::Runtime("no such device".to_string());
        Scenario::new()
            .root_layer(root_with_command(
                'c',
                command("Connect", "bluetoothctl connect", true, true),
            ))
            .run(run_layer_navigation_mode)
            .effect(navigation(&["root"]))
            .then(KeyPressed(key('c')))
            .effect(Effect::Started("bluetoothctl connect".to_string()))
            .at(100, CommandFinishes(Err(failure)))
            .effect(error("Runtime error", true))
            .then(KeyPressed(back_key()))
            .effect(navigation(&["root"]))
            .then(KeyPressed(exit_key()))
            .ends();
    }
}
