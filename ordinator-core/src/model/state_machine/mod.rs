mod state;
mod view;

use crate::model::key::KeyPress;
use crate::model::layer::Layer;
use crate::model::state_machine::state::{
    Branch, CancelTransition, Finished, GlobalKeys, Inactive, LayerActionResult,
    LayerActionTransition, ResetTransition, Root, StartTransition, State, UnbranchResult,
    UnbranchTransition,
};

use crate::model::state_machine::view::Viewable;
use crate::port::input::Input;
use crate::port::view::View;

pub enum Fsm {
    Branch(State<Branch>),
    Finished(State<Finished>),
    Inactive(State<Inactive>),
    Root(State<Root>),
}

impl Fsm {
    pub fn new(
        root: Layer,
        cancel_keys: Vec<KeyPress>,
        reset_keys: Vec<KeyPress>,
        start_keys: Vec<KeyPress>,
        unbranch_keys: Vec<KeyPress>,
    ) -> Self {
        let keys = GlobalKeys {
            cancel: cancel_keys,
            reset: reset_keys,
            start: start_keys,
            unbranch: unbranch_keys,
        };
        Fsm::Inactive(State {
            root,
            keys,
            state: Inactive,
        })
    }
}

//-----------------------------------------------------------------------------
// State machine transitions
//-----------------------------------------------------------------------------

pub trait FsmState {
    fn step(self, input: &impl Input, view: &impl View) -> Fsm;
}

impl FsmState for State<Inactive> {
    fn step(self, input: &impl Input, view: &impl View) -> Fsm {
        view.render(&self.to_view_data());
        input.capture_one(&self.keys.start);

        // Pressed key does not matter since we know it is one of the start keys.
        Fsm::Root(self.start())
    }
}

impl FsmState for State<Root> {
    fn step(self, input: &impl Input, view: &impl View) -> Fsm {
        view.render(&self.to_view_data());
        let press = input.capture_any();

        if self.keys.cancel.contains(&press) {
            Fsm::Inactive(self.cancel())
        } else {
            match self.layer_action(&press) {
                LayerActionResult::Branched(state) => Fsm::Branch(state),
                LayerActionResult::Executed(state) => Fsm::Inactive(state),
                LayerActionResult::NotFound(state) => Fsm::Root(state),
            }
        }
    }
}

impl FsmState for State<Branch> {
    fn step(self, input: &impl Input, view: &impl View) -> Fsm {
        view.render(&self.to_view_data());
        let press = input.capture_any();

        if self.keys.cancel.contains(&press) {
            Fsm::Inactive(self.cancel())
        } else if self.keys.reset.contains(&press) {
            Fsm::Root(self.reset())
        } else if self.keys.unbranch.contains(&press) {
            match self.unbranch() {
                UnbranchResult::Branch(state) => Fsm::Branch(state),
                UnbranchResult::Root(state) => Fsm::Root(state),
            }
        } else {
            match self.layer_action(&press) {
                LayerActionResult::Branched(state) => Fsm::Branch(state),
                LayerActionResult::Executed(state) => Fsm::Inactive(state),
                LayerActionResult::NotFound(state) => Fsm::Branch(state),
            }
        }
    }
}
