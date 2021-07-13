use crate::model::key::KeyPress;
use crate::model::layer::{Action, Layer};

pub struct GlobalKeys {
    pub cancel: Vec<KeyPress>,
    pub reset: Vec<KeyPress>,
    pub start: Vec<KeyPress>,
    pub unbranch: Vec<KeyPress>,
}

//-----------------------------------------------------------------------------
// States
//-----------------------------------------------------------------------------

pub struct State<S> {
    pub root: Layer,
    pub keys: GlobalKeys,
    pub state: S,
}

impl<S> State<S> {
    fn swap_state<S2>(self, state: S2) -> State<S2> {
        State {
            root: self.root,
            keys: self.keys,
            state,
        }
    }
}

pub struct Branch {
    // todo: Use non-empty list
    pub branches: Vec<Layer>,
}

pub struct Finished;

pub struct Inactive;

pub struct Root;

//-----------------------------------------------------------------------------
// Transitions
//-----------------------------------------------------------------------------

pub trait BranchTransition {
    fn branch(self, layer: Layer) -> State<Branch>;
}

impl BranchTransition for State<Branch> {
    fn branch(mut self, layer: Layer) -> State<Branch> {
        println!("Branch! Branch");
        self.state.branches.push(layer);
        self
    }
}

impl BranchTransition for State<Root> {
    fn branch(self, layer: Layer) -> State<Branch> {
        println!("Branch! Branch");
        self.swap_state(Branch {
            branches: vec![layer],
        })
    }
}

// Cancel

pub trait CancelTransition {
    fn cancel(self) -> State<Inactive>;
}

impl CancelTransition for State<Branch> {
    fn cancel(self) -> State<Inactive> {
        println!("Cancel! Inactive");
        self.swap_state(Inactive)
    }
}

impl CancelTransition for State<Root> {
    fn cancel(self) -> State<Inactive> {
        println!("Cancel! Inactive");
        self.swap_state(Inactive)
    }
}

// Execute

pub trait ExecuteTransition {
    fn execute(self) -> State<Inactive>;
}

impl ExecuteTransition for State<Branch> {
    fn execute(self) -> State<Inactive> {
        println!("Execute! Inactive");
        self.swap_state(Inactive)
    }
}

impl ExecuteTransition for State<Root> {
    fn execute(self) -> State<Inactive> {
        println!("Execute! Inactive");
        self.swap_state(Inactive)
    }
}

// Exit

pub trait ExitTransition {
    fn exit(self) -> State<Finished>;
}

impl ExitTransition for State<Branch> {
    fn exit(self) -> State<Finished> {
        println!("Exit! Finished");
        self.swap_state(Finished)
    }
}

impl ExitTransition for State<Root> {
    fn exit(self) -> State<Finished> {
        println!("Exit! Finished");
        self.swap_state(Finished)
    }
}

// Layer action

pub enum LayerActionResult<S> {
    Branched(State<Branch>),
    Executed(State<Inactive>),
    NotFound(S),
}

pub trait LayerActionTransition {
    fn layer_action(self, press: &KeyPress) -> LayerActionResult<Self>
    where
        Self: Sized;
}

impl LayerActionTransition for State<Branch> {
    fn layer_action(self, press: &KeyPress) -> LayerActionResult<Self> {
        if let Some(action) = self.state.branches.last().unwrap().actions.get(&press) {
            match action.clone() {
                Action::Branch(layer) => LayerActionResult::Branched(self.branch(layer)),
                Action::Command() => LayerActionResult::Executed(self.execute()),
            }
        } else {
            LayerActionResult::NotFound(self)
        }
    }
}

impl LayerActionTransition for State<Root> {
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
    fn reset(self) -> State<Root>;
}

impl ResetTransition for State<Branch> {
    fn reset(self) -> State<Root> {
        println!("Reset! Root");
        self.swap_state(Root)
    }
}

// Start

pub trait StartTransition {
    fn start(self) -> State<Root>;
}

impl StartTransition for State<Inactive> {
    fn start(self) -> State<Root> {
        println!("Start! Root");
        self.swap_state(Root)
    }
}

// Unbranch

pub enum UnbranchResult {
    Branch(State<Branch>),
    Root(State<Root>),
}

pub trait UnbranchTransition {
    fn unbranch(self) -> UnbranchResult;
}

impl UnbranchTransition for State<Branch> {
    fn unbranch(mut self) -> UnbranchResult {
        self.state.branches.pop();
        if self.state.branches.is_empty() {
            println!("Unbranch! Root");
            UnbranchResult::Root(self.swap_state(Root))
        } else {
            println!("Unbranch! Branch");
            UnbranchResult::Branch(self)
        }
    }
}
