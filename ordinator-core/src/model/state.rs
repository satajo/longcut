use crate::model::event::Event;
use crate::model::key::KeyPress;
use crate::model::layer::{Action, Layer};

pub enum EndCondition {
    Done,
    Exit,
}

pub struct State<'a> {
    root: &'a Layer,
    layer_stack: Vec<&'a Layer>,
}

impl<'a> State<'a> {
    pub fn new(root: &'a Layer) -> Self {
        Self {
            root,
            layer_stack: vec![],
        }
    }

    pub fn handle_keypress(mut self, input: &KeyPress) -> (Result<Self, EndCondition>, Vec<Event>) {
        match self.active_layer().actions.get(input) {
            None => (Ok(self), vec![Event::NotFound]),
            Some(action) => match action {
                Action::Branch(layer) => {
                    self.branch(layer);
                    (Ok(self), vec![Event::Branched])
                }
                Action::Command() => (Err(EndCondition::Done), vec![]),
                Action::Exit() => (Err(EndCondition::Exit), vec![Event::Exited]),
                Action::Reset() => {
                    self.reset();
                    (Ok(self), vec![Event::Reset])
                }
                Action::Unbranch() => {
                    self.unbranch();
                    (Ok(self), vec![Event::Unbranched])
                }
            },
        }
    }

    fn branch(&mut self, layer: &'a Layer) {
        self.layer_stack.push(layer);
    }

    fn reset(&mut self) {
        self.layer_stack.clear();
    }

    fn unbranch(&mut self) {
        self.layer_stack.pop();
    }

    pub fn active_layer(&self) -> &'a Layer {
        match self.layer_stack.last() {
            None => self.root,
            Some(layer) => layer,
        }
    }
}
