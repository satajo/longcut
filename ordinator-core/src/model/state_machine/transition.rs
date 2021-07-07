// Branch

use crate::model::layer::Layer;
use crate::model::state_machine::state::{Branch, Finished, Inactive, Root, State};

pub trait TransitionBranch {
    fn branch(self, layer: Layer) -> State<Branch>;
}

impl TransitionBranch for State<Branch> {
    fn branch(mut self, layer: Layer) -> State<Branch> {
        println!("Branch! Branch");
        self.state.branches.push(layer);
        self
    }
}

impl TransitionBranch for State<Root> {
    fn branch(self, layer: Layer) -> State<Branch> {
        println!("Branch! Branch");
        self.swap_state(Branch {
            branches: vec![layer],
        })
    }
}

// Cancel

pub trait TransitionCancel {
    fn cancel(self) -> State<Inactive>;
}

impl TransitionCancel for State<Branch> {
    fn cancel(self) -> State<Inactive> {
        println!("Cancel! Inactive");
        self.swap_state(Inactive)
    }
}

impl TransitionCancel for State<Root> {
    fn cancel(self) -> State<Inactive> {
        println!("Cancel! Inactive");
        self.swap_state(Inactive)
    }
}

// Execute

pub trait TransitionExecute {
    fn execute(self) -> State<Inactive>;
}

impl TransitionExecute for State<Branch> {
    fn execute(self) -> State<Inactive> {
        println!("Execute! Inactive");
        self.swap_state(Inactive)
    }
}

impl TransitionExecute for State<Root> {
    fn execute(self) -> State<Inactive> {
        println!("Execute! Inactive");
        self.swap_state(Inactive)
    }
}

// Exit

pub trait TransitionExit {
    fn exit(self) -> State<Finished>;
}

impl TransitionExit for State<Branch> {
    fn exit(self) -> State<Finished> {
        println!("Exit! Finished");
        self.swap_state(Finished)
    }
}

impl TransitionExit for State<Root> {
    fn exit(self) -> State<Finished> {
        println!("Exit! Finished");
        self.swap_state(Finished)
    }
}

// Reset

pub trait TransitionReset {
    fn reset(self) -> State<Root>;
}

impl TransitionReset for State<Branch> {
    fn reset(self) -> State<Root> {
        println!("Reset! Root");
        self.swap_state(Root)
    }
}

// Start

pub trait TransitionStart {
    fn start(self) -> State<Root>;
}

impl TransitionStart for State<Inactive> {
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

pub trait TransitionUnbranch {
    fn unbranch(self) -> UnbranchResult;
}

impl TransitionUnbranch for State<Branch> {
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
