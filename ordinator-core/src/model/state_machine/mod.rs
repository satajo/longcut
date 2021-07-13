mod state;
mod transition;
mod view;

use crate::model::key::KeyPress;
use crate::model::layer::{Action, Layer};
use crate::model::state_machine::state::{Branch, Finished, GlobalKeys, Inactive, Root, State};
use crate::model::state_machine::transition::{
    TransitionBranch, TransitionCancel, TransitionExecute, TransitionReset, TransitionStart,
    TransitionUnbranch, UnbranchResult,
};
use crate::model::state_machine::view::Viewable;
use crate::port::input::Input;
use crate::port::view::{View, ViewData};

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

        // Pressed key does not matter since we know it is one of the start keys.
        input.capture_one(&self.keys.start);

        let state = self.start();
        Fsm::Root(state)
    }
}

impl FsmState for State<Root> {
    fn step(self, input: &impl Input, view: &impl View) -> Fsm {
        view.render(&self.to_view_data());

        let press = input.capture_any();

        // Global keys
        if self.keys.cancel.contains(&press) {
            Fsm::Inactive(self.cancel())

        // Layer actions
        } else if let Some(action) = self.root.actions.get(&press) {
            match action.clone() {
                Action::Branch(layer) => Fsm::Branch(self.branch(layer)),
                Action::Command() => Fsm::Inactive(self.execute()),
            }

        // No match
        } else {
            Fsm::Root(self)
        }
    }
}

impl FsmState for State<Branch> {
    fn step(self, input: &impl Input, view: &impl View) -> Fsm {
        view.render(&self.to_view_data());

        let press = input.capture_any();

        // Global keys
        if self.keys.cancel.contains(&press) {
            Fsm::Inactive(self.cancel())
        } else if self.keys.reset.contains(&press) {
            Fsm::Root(self.reset())
        } else if self.keys.unbranch.contains(&press) {
            match self.unbranch() {
                UnbranchResult::Branch(state) => Fsm::Branch(state),
                UnbranchResult::Root(state) => Fsm::Root(state),
            }

        // Layer actions
        } else if let Some(action) = self.state.branches.last().unwrap().actions.get(&press) {
            match action.clone() {
                Action::Branch(layer) => Fsm::Branch(self.branch(layer)),
                Action::Command() => Fsm::Inactive(self.execute()),
            }

        // No match
        } else {
            Fsm::Branch(self)
        }
    }
}
