use crate::logic::command_execution::{CommandExecutionProgram, ProgramResult};
use crate::model::key::Key;
use crate::model::layer::{Action, Layer};
use crate::port::input::Input;
use crate::port::view::{LayerNavigationViewModel, View, ViewAction, ViewModel};
use std::ops::Deref;

pub struct LayerStackProgram<'a> {
    input: &'a dyn Input,
    view: &'a dyn View,
    // Configuration
    command_executor: &'a CommandExecutionProgram<'a>,
    keys_back: &'a [Key],
    keys_deactivate: &'a [Key],
    root_layer: &'a Layer,
}

impl<'a> LayerStackProgram<'a> {
    pub fn new(
        input: &'a dyn Input,
        view: &'a dyn View,
        command_executor: &'a CommandExecutionProgram,
        keys_back: &'a [Key],
        keys_deactivate: &'a [Key],
        root_layer: &'a Layer,
    ) -> Self {
        Self {
            input,
            view,
            command_executor,
            keys_back,
            keys_deactivate,
            root_layer,
        }
    }

    pub fn run(&self) {
        let mut layers = vec![self.root_layer];
        loop {
            let active_layer = layers.last().unwrap();
            let is_branched = layers.len() > 1;

            // Rendering
            if is_branched {
                self.render_branch(layers.as_slice());
            } else {
                self.render_root(active_layer);
            }

            // Input handling
            let press = self.input.capture_any();
            if self.keys_deactivate.contains(&press) {
                return;
            }

            if is_branched && self.keys_back.contains(&press) {
                layers.pop();
                continue;
            }

            if let Some(action) = active_layer.resolve_shortcut(&press) {
                match action {
                    Action::Branch(into) => {
                        layers.push(into);
                    }
                    Action::Execute(command) => {
                        match self.command_executor.run(command, &layers) {
                            ProgramResult::KeepGoing => {
                                // Do nothing.
                            }

                            ProgramResult::Finished => {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    fn render_root(&self, layer: &Layer) {
        let mut actions = Self::render_layer_actions(layer);

        // Deactivate is always available.
        for key in self.keys_deactivate {
            actions.push((key, ViewAction::Deactivate));
        }

        self.render_navigation_view(actions, &[layer]);
    }

    fn render_branch(&self, layers: &[&Layer]) {
        let mut actions = Self::render_layer_actions(layers.last().unwrap());

        // Back keys are available.
        for key in self.keys_back {
            actions.push((key, ViewAction::Unbranch));
        }

        // Deactivate is always available.
        for key in self.keys_deactivate {
            actions.push((key, ViewAction::Deactivate));
        }

        self.render_navigation_view(actions, layers);
    }

    fn render_navigation_view(&self, actions: Vec<(&Key, ViewAction)>, layers: &[&Layer]) {
        let model = LayerNavigationViewModel {
            actions: &actions,
            layer_stack: layers,
        };

        self.view.render(ViewModel::LayerNavigation(model));
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
}
