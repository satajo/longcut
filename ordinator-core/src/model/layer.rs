use crate::model::command::Command;
use crate::model::key::Key;
use crate::model::shortcut_map::ShortcutMap;

#[derive(Debug)]
pub enum Action {
    Branch(Layer),
    Execute(Command),
}

#[derive(Debug)]
pub struct Layer {
    pub name: String,
    pub shortcuts: ShortcutMap<Action>,
}

impl Layer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            shortcuts: ShortcutMap::new(),
        }
    }

    pub fn add_command(&mut self, shortcut: Key, command: Command) -> Result<(), (Key, Action)> {
        self.shortcuts
            .try_assign(shortcut, Action::Execute(command))
    }

    pub fn add_layer(&mut self, shortcut: Key, layer: Layer) -> Result<(), (Key, Action)> {
        self.shortcuts.try_assign(shortcut, Action::Branch(layer))
    }

    pub fn resolve_shortcut(&self, key: &Key) -> Option<&Action> {
        self.shortcuts.match_fuzzy(key)
    }
}
