use crate::model::key::KeyPress;
use crate::model::layer::{Action, Layer};
use crate::port::view::{ToViewData, ViewAction, ViewData};

//-----------------------------------------------------------------------------
// States
//-----------------------------------------------------------------------------

pub struct StateMachine<S> {
    pub root: Layer,
    pub state: S,
}

pub struct Branch {
    // todo: Use non-empty list
    pub layers: Vec<Layer>,
}

pub struct Finished;

pub struct Inactive;

pub struct Root;

impl<S> StateMachine<S> {
    fn swap_state<S2>(self, state: S2) -> StateMachine<S2> {
        StateMachine {
            root: self.root,
            state,
        }
    }
}

impl StateMachine<Inactive> {
    pub fn new(root: Layer) -> Self {
        Self {
            root,
            state: Inactive,
        }
    }
}

//-----------------------------------------------------------------------------
// Transitions
//-----------------------------------------------------------------------------

pub trait BranchTransition {
    fn branch(self, layer: Layer) -> StateMachine<Branch>;
}

impl BranchTransition for StateMachine<Branch> {
    fn branch(mut self, layer: Layer) -> StateMachine<Branch> {
        println!("Branch! Branch");
        self.state.layers.push(layer);
        self
    }
}

impl BranchTransition for StateMachine<Root> {
    fn branch(self, layer: Layer) -> StateMachine<Branch> {
        println!("Branch! Branch");
        self.swap_state(Branch {
            layers: vec![layer],
        })
    }
}

// Cancel

pub trait CancelTransition {
    fn cancel(self) -> StateMachine<Inactive>;
}

impl CancelTransition for StateMachine<Branch> {
    fn cancel(self) -> StateMachine<Inactive> {
        println!("Cancel! Inactive");
        self.swap_state(Inactive)
    }
}

impl CancelTransition for StateMachine<Root> {
    fn cancel(self) -> StateMachine<Inactive> {
        println!("Cancel! Inactive");
        self.swap_state(Inactive)
    }
}

// Execute

pub trait ExecuteTransition {
    fn execute(self) -> StateMachine<Inactive>;
}

impl ExecuteTransition for StateMachine<Branch> {
    fn execute(self) -> StateMachine<Inactive> {
        println!("Execute! Inactive");
        self.swap_state(Inactive)
    }
}

impl ExecuteTransition for StateMachine<Root> {
    fn execute(self) -> StateMachine<Inactive> {
        println!("Execute! Inactive");
        self.swap_state(Inactive)
    }
}

// Exit

pub trait ExitTransition {
    fn exit(self) -> StateMachine<Finished>;
}

impl ExitTransition for StateMachine<Branch> {
    fn exit(self) -> StateMachine<Finished> {
        println!("Exit! Finished");
        self.swap_state(Finished)
    }
}

impl ExitTransition for StateMachine<Root> {
    fn exit(self) -> StateMachine<Finished> {
        println!("Exit! Finished");
        self.swap_state(Finished)
    }
}

// Layer action

pub enum LayerActionResult<S> {
    Branched(StateMachine<Branch>),
    Executed(StateMachine<Inactive>),
    NotFound(S),
}

pub trait LayerActionTransition {
    fn layer_action(self, press: &KeyPress) -> LayerActionResult<Self>
    where
        Self: Sized;
}

impl LayerActionTransition for StateMachine<Branch> {
    fn layer_action(self, press: &KeyPress) -> LayerActionResult<Self> {
        if let Some(action) = self.state.layers.last().unwrap().actions.get(&press) {
            match action.clone() {
                Action::Branch(layer) => LayerActionResult::Branched(self.branch(layer)),
                Action::Command() => LayerActionResult::Executed(self.execute()),
            }
        } else {
            LayerActionResult::NotFound(self)
        }
    }
}

impl LayerActionTransition for StateMachine<Root> {
    fn layer_action(self, press: &KeyPress) -> LayerActionResult<Self> {
        if let Some(action) = self.root.actions.get(&press) {
            match action.clone() {
                Action::Branch(layer) => LayerActionResult::Branched(self.branch(layer)),
                Action::Command() => LayerActionResult::Executed(self.execute()),
            }
        } else {
            LayerActionResult::NotFound(self)
        }
    }
}

// Reset

pub trait ResetTransition {
    fn reset(self) -> StateMachine<Root>;
}

impl ResetTransition for StateMachine<Branch> {
    fn reset(self) -> StateMachine<Root> {
        println!("Reset! Root");
        self.swap_state(Root)
    }
}

// Start

pub trait StartTransition {
    fn start(self) -> StateMachine<Root>;
}

impl StartTransition for StateMachine<Inactive> {
    fn start(self) -> StateMachine<Root> {
        println!("Start! Root");
        self.swap_state(Root)
    }
}

// Unbranch

pub enum UnbranchResult {
    Branch(StateMachine<Branch>),
    Root(StateMachine<Root>),
}

pub trait UnbranchTransition {
    fn unbranch(self) -> UnbranchResult;
}

impl UnbranchTransition for StateMachine<Branch> {
    fn unbranch(mut self) -> UnbranchResult {
        self.state.layers.pop();
        if self.state.layers.is_empty() {
            println!("Unbranch! Root");
            UnbranchResult::Root(self.swap_state(Root))
        } else {
            println!("Unbranch! Branch");
            UnbranchResult::Branch(self)
        }
    }
}

//-----------------------------------------------------------------------------
// View model mapping
//-----------------------------------------------------------------------------

impl ToViewData for StateMachine<Branch> {
    fn to_view_data(&self) -> ViewData {
        let mut actions = vec![];
        for (press, action) in &self.state.layers.last().unwrap().actions {
            let view_action = match action {
                Action::Branch(layer) => ViewAction::Branch(layer.name.clone()),
                Action::Command() => ViewAction::Execute("".to_string()),
            };

            actions.push((press.clone(), view_action))
        }

        let mut layers = vec![self.root.name.clone()];
        for layer in &self.state.layers {
            layers.push(layer.name.clone())
        }

        ViewData {
            visible: true,
            actions,
            layers,
        }
    }
}

impl ToViewData for StateMachine<Root> {
    fn to_view_data(&self) -> ViewData {
        let mut actions = vec![];
        for (press, action) in &self.root.actions {
            let view_action = match action {
                Action::Branch(layer) => ViewAction::Branch(layer.name.clone()),
                Action::Command() => ViewAction::Execute("".to_string()),
            };

            actions.push((press.clone(), view_action))
        }

        ViewData {
            visible: true,
            actions,
            layers: vec![self.root.name.clone()],
        }
    }
}
