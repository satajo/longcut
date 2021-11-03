use crate::logic::command_execution::CommandExecutionProgram;
use crate::logic::Context;
use crate::model::layer::{Action, Layer};
use crate::port::view::{LayerViewData, ViewAction, ViewState};

pub struct LayerStackProgram;

impl LayerStackProgram {
    pub fn run(ctx: &Context) {
        let mut layers = vec![ctx.root_layer];
        loop {
            let active_layer = layers.last().unwrap();
            let is_branched = layers.len() > 1;

            // Rendering
            if is_branched {
                Self::render_branch(ctx, layers.as_slice());
            } else {
                Self::render_root(ctx, active_layer);
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

            if let Some(action) = active_layer.actions.get(&press) {
                match action {
                    Action::Branch(into) => {
                        layers.push(into);
                    }
                    Action::Command() => {
                        CommandExecutionProgram::run(&ctx);
                        return;
                    }
                }
            }
        }
    }

    fn render_root(ctx: &Context, layer: &Layer) {
        let mut actions = vec![];

        // Collecting all layer actions into the view action vector.
        for (press, action) in &layer.actions {
            let view_action = match action {
                Action::Branch(layer) => ViewAction::Branch(layer.name.clone()),
                Action::Command() => ViewAction::Execute("".to_string()),
            };

            actions.push((press.clone(), view_action))
        }

        // Deactivate is always available.
        for key in ctx.keys_deactivate {
            actions.push((key.clone(), ViewAction::Deactivate()));
        }

        // Rendering
        let data = LayerViewData {
            actions,
            layers: vec![layer.name.clone()],
        };

        ctx.view.render(&ViewState::LayerView(data));
    }

    fn render_branch(ctx: &Context, layers: &[&Layer]) {
        let mut actions = vec![];

        // Collecting all layer actions into the view action vector.
        for (press, action) in &layers.last().unwrap().actions {
            let view_action = match action {
                Action::Branch(layer) => ViewAction::Branch(layer.name.clone()),
                Action::Command() => ViewAction::Execute("".to_string()),
            };

            actions.push((press.clone(), view_action))
        }

        // Back keys are available.
        for key in ctx.keys_back {
            actions.push((key.clone(), ViewAction::Unbranch()));
        }

        // Deactivate is always available.
        for key in ctx.keys_deactivate {
            actions.push((key.clone(), ViewAction::Deactivate()));
        }

        // Rendering
        let data = LayerViewData {
            actions,
            layers: layers.iter().map(|layer| layer.name.clone()).collect(),
        };
        ctx.view.render(&ViewState::LayerView(data));
    }
}
