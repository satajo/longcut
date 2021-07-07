use crate::model::key::KeyPress;
use crate::model::layer::Layer;

pub struct GlobalKeys {
    pub cancel: Vec<KeyPress>,
    pub reset: Vec<KeyPress>,
    pub start: Vec<KeyPress>,
    pub unbranch: Vec<KeyPress>,
}

//-----------------------------------------------------------------------------
// Common state structure
//-----------------------------------------------------------------------------

pub struct State<S> {
    pub root: Layer,
    pub keys: GlobalKeys,
    pub state: S,
}

impl<S> State<S> {
    pub fn swap_state<S2>(mut self, state: S2) -> State<S2> {
        State {
            root: self.root,
            keys: self.keys,
            state,
        }
    }
}

//-----------------------------------------------------------------------------
// State declaration
//-----------------------------------------------------------------------------

pub struct Branch {
    // todo: Use non-empty list
    pub branches: Vec<Layer>,
}

pub struct Finished;

pub struct Inactive;

pub struct Root;
