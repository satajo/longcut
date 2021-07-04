use crate::model::event::Event;
use crate::model::key::KeyPress;
use crate::model::layer::{Action, Layer};
use std::sync::atomic::Ordering::SeqCst;

pub struct GlobalKeys {
    pub cancel: Vec<KeyPress>,
    pub start: Vec<KeyPress>,
    pub unbranch: Vec<KeyPress>,
    pub exit: Vec<KeyPress>,
}

//-----------------------------------------------------------------------------
// Initial state
//-----------------------------------------------------------------------------

pub struct InitialState {
    root: Layer,
    global_keys: GlobalKeys,
}

impl InitialState {
    pub fn new(root: Layer, global_keys: GlobalKeys) -> Self {
        Self { root, global_keys }
    }

    pub fn begin_sequence(&self) -> Sequence {
        Sequence::new(&self.root, &self.global_keys)
    }

    pub fn launch_keys(&self) -> &Vec<KeyPress> {
        &self.global_keys.start
    }
}

pub struct Sequence<'a> {
    global_keys: &'a GlobalKeys,
    layer_stack: Vec<&'a Layer>,
    root: &'a Layer,
}

pub enum SequenceState<'a> {
    Active(Sequence<'a>),
    Done,
    Exit,
}

impl<'a> Sequence<'a> {
    pub fn new(root: &'a Layer, global_keys: &'a GlobalKeys) -> Self {
        Self {
            root,
            global_keys,
            layer_stack: vec![],
        }
    }

    pub fn handle_keypress(mut self, input: &KeyPress) -> (SequenceState<'a>, Vec<Event>) {
        if self.global_keys.exit.contains(input) {
            (SequenceState::Exit, vec![Event::Exited])
        } else if self.global_keys.cancel.contains(input) {
            (SequenceState::Done, vec![Event::Canceled])
        } else if self.global_keys.unbranch.contains(input) {
            self.unbranch();
            (SequenceState::Active(self), vec![Event::Unbranched])
        } else if let Some(action) = self.active_layer().actions.get(input) {
            match action {
                Action::Branch(layer) => {
                    self.branch(layer);
                    (SequenceState::Active(self), vec![Event::Branched])
                }
                Action::Command() => (SequenceState::Done, vec![]),
            }
        } else {
            (SequenceState::Active(self), vec![Event::NotFound])
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
