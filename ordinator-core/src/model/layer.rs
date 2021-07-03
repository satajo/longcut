use std::collections::BTreeMap;

use crate::model::key::KeyPress;

#[derive(Clone)]
pub enum Action {
    Branch(Layer),
    Command(),
    Exit(),
    Reset(),
    Unbranch(),
}

#[derive(Clone)]
pub struct Layer {
    pub name: String,
    pub actions: BTreeMap<KeyPress, Action>,
}

impl Layer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            actions: BTreeMap::new(),
        }
    }

    pub fn add_action(mut self, shortcut: KeyPress, action: Action) -> Self {
        self.actions.insert(shortcut, action);
        return self;
    }
}
