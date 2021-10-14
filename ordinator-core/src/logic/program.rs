use super::state_machine::{
    Branch, CancelTransition, Inactive, LayerActionResult, LayerActionTransition, Root,
    StartTransition, StateMachine, UnbranchResult, UnbranchTransition,
};
use crate::model::key::KeyPress;
use crate::port::input::Input;
use crate::port::view::{ToViewData, View, ViewAction, ViewData};

pub struct ProgramState<'a, S> {
    state: S,
    // Dependencies
    input: &'a dyn Input,
    view: &'a dyn View,
    // Configuration
    keys_activate: Vec<KeyPress>,
    keys_back: Vec<KeyPress>,
    keys_deactivate: Vec<KeyPress>,
}

impl<'a, S> ProgramState<'a, S> {
    fn map_state<S2>(self, f: impl FnOnce(S) -> S2) -> ProgramState<'a, S2> {
        ProgramState {
            state: f(self.state),
            input: self.input,
            view: self.view,
            keys_activate: self.keys_activate,
            keys_back: self.keys_back,
            keys_deactivate: self.keys_deactivate,
        }
    }
}

pub enum Program<'a> {
    Branch(ProgramState<'a, StateMachine<'a, Branch<'a>>>),
    Inactive(ProgramState<'a, StateMachine<'a, Inactive>>),
    Root(ProgramState<'a, StateMachine<'a, Root>>),
}

impl<'a> Program<'a> {
    pub fn new(
        input: &'a impl Input,
        view: &'a impl View,
        initial_state: StateMachine<'a, Inactive>,
        keys_activate: Vec<KeyPress>,
        keys_back: Vec<KeyPress>,
        keys_deactivate: Vec<KeyPress>,
    ) -> Self {
        Self::Inactive(ProgramState {
            state: initial_state,
            input,
            view,
            keys_activate,
            keys_back,
            keys_deactivate,
        })
    }
}

//-----------------------------------------------------------------------------
// Operations
//-----------------------------------------------------------------------------

pub trait RunProgram<'a> {
    fn run(self) -> Program<'a>;
}

impl<'a> RunProgram<'a> for ProgramState<'a, StateMachine<'a, Inactive>> {
    fn run(self) -> Program<'a> {
        self.view.render(&self.to_view_data());
        self.input.capture_one(&self.keys_activate);

        // Pressed key does not matter since we know it is one of the start keys.
        Program::Root(self.map_state(|s| s.start()))
    }
}

impl<'a> RunProgram<'a> for ProgramState<'a, StateMachine<'a, Root>> {
    fn run(self) -> Program<'a> {
        self.view.render(&self.to_view_data());
        let press = self.input.capture_any();

        if self.keys_deactivate.contains(&press) {
            Program::Inactive(self.map_state(|s| s.cancel()))
        } else {
            match self.state.layer_action(&press) {
                LayerActionResult::Branched(state) => Program::Branch(ProgramState {
                    state,
                    input: self.input,
                    view: self.view,
                    keys_activate: self.keys_activate,
                    keys_back: self.keys_back,
                    keys_deactivate: self.keys_deactivate,
                }),
                LayerActionResult::Executed(state) => Program::Inactive(ProgramState {
                    state,
                    input: self.input,
                    view: self.view,
                    keys_activate: self.keys_activate,
                    keys_back: self.keys_back,
                    keys_deactivate: self.keys_deactivate,
                }),
                LayerActionResult::NotFound(state) => Program::Root(ProgramState {
                    state,
                    input: self.input,
                    view: self.view,
                    keys_activate: self.keys_activate,
                    keys_back: self.keys_back,
                    keys_deactivate: self.keys_deactivate,
                }),
            }
        }
    }
}

impl<'a> RunProgram<'a> for ProgramState<'a, StateMachine<'a, Branch<'a>>> {
    fn run(self) -> Program<'a> {
        self.view.render(&self.to_view_data());
        let press = self.input.capture_any();

        if self.keys_deactivate.contains(&press) {
            Program::Inactive(self.map_state(|s| s.cancel()))
        } else if self.keys_back.contains(&press) {
            match self.state.unbranch() {
                UnbranchResult::Branch(state) => Program::Branch(ProgramState {
                    state,
                    input: self.input,
                    view: self.view,
                    keys_activate: self.keys_activate,
                    keys_back: self.keys_back,
                    keys_deactivate: self.keys_deactivate,
                }),
                UnbranchResult::Root(state) => Program::Root(ProgramState {
                    state,
                    input: self.input,
                    view: self.view,
                    keys_activate: self.keys_activate,
                    keys_back: self.keys_back,
                    keys_deactivate: self.keys_deactivate,
                }),
            }
        } else {
            match self.state.layer_action(&press) {
                LayerActionResult::Branched(state) => Program::Branch(ProgramState {
                    state,
                    input: self.input,
                    view: self.view,
                    keys_activate: self.keys_activate,
                    keys_back: self.keys_back,
                    keys_deactivate: self.keys_deactivate,
                }),
                LayerActionResult::Executed(state) => Program::Inactive(ProgramState {
                    state,
                    input: self.input,
                    view: self.view,
                    keys_activate: self.keys_activate,
                    keys_back: self.keys_back,
                    keys_deactivate: self.keys_deactivate,
                }),
                LayerActionResult::NotFound(state) => Program::Branch(ProgramState {
                    state,
                    input: self.input,
                    view: self.view,
                    keys_activate: self.keys_activate,
                    keys_back: self.keys_back,
                    keys_deactivate: self.keys_deactivate,
                }),
            }
        }
    }
}

//-----------------------------------------------------------------------------
// View model transformations
//-----------------------------------------------------------------------------

pub trait Viewable {
    fn to_view_data(&self) -> ViewData;
}

impl<'a> Viewable for ProgramState<'a, StateMachine<'a, Inactive>> {
    fn to_view_data(&self) -> ViewData {
        ViewData {
            visible: false,
            actions: vec![],
            layers: vec![],
        }
    }
}

impl<'a> Viewable for ProgramState<'a, StateMachine<'a, Branch<'a>>> {
    fn to_view_data(&self) -> ViewData {
        let mut view_data = self.state.to_view_data();

        // Program state specific actions are added to the view data.
        for key in &self.keys_back {
            view_data
                .actions
                .push((key.clone(), ViewAction::Unbranch()));
        }

        for key in &self.keys_deactivate {
            view_data
                .actions
                .push((key.clone(), ViewAction::Deactivate()));
        }

        view_data
    }
}

impl<'a> Viewable for ProgramState<'a, StateMachine<'a, Root>> {
    fn to_view_data(&self) -> ViewData {
        let mut view_data = self.state.to_view_data();

        // Program state specific actions are added to the view data.
        for key in &self.keys_deactivate {
            view_data
                .actions
                .push((key.clone(), ViewAction::Deactivate()));
        }

        view_data
    }
}
