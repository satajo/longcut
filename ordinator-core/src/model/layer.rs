use std::collections::BTreeMap;

use crate::model::effect::Effect;
use crate::model::key::KeyPress;

#[derive(Clone)]
pub struct Layer {
    pub name: String,
    pub actions: BTreeMap<KeyPress, Vec<Effect>>,
}

impl Layer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            actions: BTreeMap::new(),
        }
    }

    pub fn add_action(mut self, shortcut: KeyPress, effects: Vec<Effect>) -> Self {
        self.actions.insert(shortcut, effects);
        return self;
    }
}
