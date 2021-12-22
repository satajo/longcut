use crate::model::command::Command;
use crate::model::key::Key;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Action {
    Branch(Layer),
    Execute(Command),
}

#[derive(Debug)]
pub struct Layer {
    pub name: String,
    pub shortcuts: BTreeMap<Key, Action>,
}

impl Layer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            shortcuts: BTreeMap::new(),
        }
    }

    fn try_add_shortcut(&mut self, shortcut: Key, action: Action) -> Result<(), (Key, Action)> {
        match self.shortcuts.get(&shortcut) {
            None => {
                self.shortcuts.insert(shortcut, action);
                Ok(())
            }
            Some(_) => Err((shortcut, action)),
        }
    }

    pub fn add_command(&mut self, shortcut: Key, command: Command) -> Result<(), (Key, Action)> {
        self.try_add_shortcut(shortcut, Action::Execute(command))
    }

    pub fn add_layer(&mut self, shortcut: Key, layer: Layer) -> Result<(), (Key, Action)> {
        self.try_add_shortcut(shortcut, Action::Branch(layer))
    }

    pub fn resolve_shortcut(&self, key: &Key) -> Option<&Action> {
        self.shortcuts.get(key)
    }
}
