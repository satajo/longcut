use crate::model::key::KeyPress;
use crate::model::layer::{Action, Layer};
use crate::port::view::{ToViewData, ViewAction, ViewData};

//-----------------------------------------------------------------------------
// States
//-----------------------------------------------------------------------------

pub struct StateMachine<'a, S> {
    pub root: &'a Layer,
    pub state: S,
}

pub struct Branch<'a> {
    // todo: Use non-empty list
    pub layers: Vec<&'a Layer>,
}

pub struct Finished;

pub struct Inactive;

pub struct Root;

impl<'a, S> StateMachine<'a, S> {
    fn swap_state<S2>(self, state: S2) -> StateMachine<'a, S2> {
        StateMachine {
            root: self.root,
            state,
        }
    }
}

impl<'a> StateMachine<'a, Inactive> {
    pub fn new(root: &'a Layer) -> Self {
        Self {
            root,
            state: Inactive,
        }
    }
}

//-----------------------------------------------------------------------------
// Transitions
//-----------------------------------------------------------------------------

pub trait BranchTransition<'a> {
    fn branch(self, layer: &'a Layer) -> StateMachine<'a, Branch<'a>>;
}

impl<'a> BranchTransition<'a> for StateMachine<'a, Branch<'a>> {
    fn branch(mut self, layer: &'a Layer) -> StateMachine<'a, Branch<'a>> {
        println!("Branch! Branch");
        self.state.layers.push(layer);
        self
    }
}

impl<'a> BranchTransition<'a> for StateMachine<'a, Root> {
    fn branch(self, layer: &'a Layer) -> StateMachine<'a, Branch<'a>> {
        println!("Branch! Branch");
        self.swap_state(Branch {
            layers: vec![layer],
        })
    }
}

// Cancel

pub trait CancelTransition<'a> {
    fn cancel(self) -> StateMachine<'a, Inactive>;
}

impl<'a> CancelTransition<'a> for StateMachine<'a, Branch<'a>> {
    fn cancel(self) -> StateMachine<'a, Inactive> {
        println!("Cancel! Inactive");
        self.swap_state(Inactive)
    }
}

impl<'a> CancelTransition<'a> for StateMachine<'a, Root> {
    fn cancel(self) -> StateMachine<'a, Inactive> {
        println!("Cancel! Inactive");
        self.swap_state(Inactive)
    }
}

// Execute

pub trait ExecuteTransition<'a> {
    fn execute(self) -> StateMachine<'a, Inactive>;
}

impl<'a> ExecuteTransition<'a> for StateMachine<'a, Branch<'a>> {
    fn execute(self) -> StateMachine<'a, Inactive> {
        println!("Execute! Inactive");
        self.swap_state(Inactive)
    }
}

impl<'a> ExecuteTransition<'a> for StateMachine<'a, Root> {
    fn execute(self) -> StateMachine<'a, Inactive> {
        println!("Execute! Inactive");
        self.swap_state(Inactive)
    }
}

// Exit

pub trait ExitTransition<'a> {
    fn exit(self) -> StateMachine<'a, Finished>;
}

impl<'a> ExitTransition<'a> for StateMachine<'a, Branch<'a>> {
    fn exit(self) -> StateMachine<'a, Finished> {
        println!("Exit! Finished");
        self.swap_state(Finished)
    }
}

impl<'a> ExitTransition<'a> for StateMachine<'a, Root> {
    fn exit(self) -> StateMachine<'a, Finished> {
        println!("Exit! Finished");
        self.swap_state(Finished)
    }
}

// Layer action

pub enum LayerActionResult<'a, S> {
    Branched(StateMachine<'a, Branch<'a>>),
    Executed(StateMachine<'a, Inactive>),
    NotFound(S),
}

pub trait LayerActionTransition<'a> {
    fn layer_action(self, press: &KeyPress) -> LayerActionResult<'a, Self>
    where
        Self: Sized;
}

impl<'a> LayerActionTransition<'a> for StateMachine<'a, Branch<'a>> {
    fn layer_action(self, press: &KeyPress) -> LayerActionResult<'a, Self> {
        if let Some(action) = self.state.layers.last().unwrap().actions.get(press) {
            match action {
                Action::Branch(layer) => LayerActionResult::Branched(self.branch(layer)),
                Action::Command() => LayerActionResult::Executed(self.execute()),
            }
        } else {
            LayerActionResult::NotFound(self)
        }
    }
}

impl<'a> LayerActionTransition<'a> for StateMachine<'a, Root> {
    fn layer_action(self, press: &KeyPress) -> LayerActionResult<'a, Self> {
        if let Some(action) = self.root.actions.get(press) {
            match action {
                Action::Branch(layer) => LayerActionResult::Branched(self.branch(layer)),
                Action::Command() => LayerActionResult::Executed(self.execute()),
            }
        } else {
            LayerActionResult::NotFound(self)
        }
    }
}

// Reset

pub trait ResetTransition<'a> {
    fn reset(self) -> StateMachine<'a, Root>;
}

impl<'a> ResetTransition<'a> for StateMachine<'a, Branch<'a>> {
    fn reset(self) -> StateMachine<'a, Root> {
        println!("Reset! Root");
        self.swap_state(Root)
    }
}

// Start

pub trait StartTransition<'a> {
    fn start(self) -> StateMachine<'a, Root>;
}

impl<'a> StartTransition<'a> for StateMachine<'a, Inactive> {
    fn start(self) -> StateMachine<'a, Root> {
        println!("Start! Root");
        self.swap_state(Root)
    }
}

// Unbranch

pub enum UnbranchResult<'a> {
    Branch(StateMachine<'a, Branch<'a>>),
    Root(StateMachine<'a, Root>),
}

pub trait UnbranchTransition<'a> {
    fn unbranch(self) -> UnbranchResult<'a>;
}

impl<'a> UnbranchTransition<'a> for StateMachine<'a, Branch<'a>> {
    fn unbranch(mut self) -> UnbranchResult<'a> {
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

impl<'a> ToViewData for StateMachine<'a, Branch<'a>> {
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

impl<'a> ToViewData for StateMachine<'a, Root> {
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
