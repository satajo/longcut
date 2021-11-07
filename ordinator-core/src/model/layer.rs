use crate::model::command::Command;
use crate::model::key::KeyPress;
use std::collections::btree_map::Entry;
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

    fn try_add_shortcut(
        &mut self,
        shortcut: KeyPress,
        action: Action,
    ) -> Result<(), (KeyPress, Action)> {
        match self.shortcuts.get(&shortcut) {
            None => {
                self.shortcuts.insert(shortcut, action);
                Ok(())
            }
            Some(_) => Err((shortcut, action)),
        }
    }

    pub fn add_command(
        &mut self,
        shortcut: KeyPress,
        command: Command,
    ) -> Result<(), (KeyPress, Action)> {
        self.try_add_shortcut(shortcut, Action::Execute(command))
    }

    pub fn add_layer(
        &mut self,
        shortcut: KeyPress,
        layer: Layer,
    ) -> Result<(), (KeyPress, Action)> {
        self.try_add_shortcut(shortcut, Action::Branch(layer))
    }

    pub fn resolve_shortcut(&self, key: &KeyPress) -> Option<&Action> {
        self.shortcuts.get(key)
    }
}
