use crate::model::command::Command;
use crate::model::key::KeyPress;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Action {
    Branch(Layer),
    Execute(Command),
}

#[derive(Debug)]
pub struct Layer {
    pub name: String,
    pub shortcuts: BTreeMap<KeyPress, Action>,
}

impl Layer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            shortcuts: BTreeMap::new(),
        }
    }

    pub fn add_command(&mut self, shortcut: KeyPress, command: Command) {
        self.shortcuts.insert(shortcut, Action::Execute(command));
    }

    pub fn add_layer(&mut self, shortcut: KeyPress, layer: Layer) {
        self.shortcuts.insert(shortcut, Action::Branch(layer));
    }

    pub fn resolve_shortcut(&self, key: &KeyPress) -> Option<&Action> {
        self.shortcuts.get(key)
    }
}
