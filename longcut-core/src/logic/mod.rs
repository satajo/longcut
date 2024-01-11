/// Modes represent the different states that the program can be in.
///
/// The modes operate independently, handing over control to other modes when required. The modes
/// form a tree structure, being aware of the modes which they depend upon, but not of the modes
/// which depend on them.
mod command_execution;
mod error;
mod inactive;
mod layer_navigation;
mod parameter_input;

use self::inactive::run_inactive_mode;
use crate::model::key::Key;
use crate::model::layer::Layer;
use crate::port::executor::Executor;
use crate::port::input::Input;
use crate::port::view::View;

/// Context is the container for the shared configuration and dependencies of the mode logic.
pub struct Context<'a> {
    pub executor: &'a dyn Executor,
    pub input: &'a dyn Input,
    pub view: &'a dyn View,

    // Configuration
    pub keys_activate: &'a [Key],
    pub keys_back: &'a [Key],
    pub keys_deactivate: &'a [Key],
    pub keys_retry: &'a [Key],

    // Layer
    pub root_layer: &'a Layer,
}

/// The main entrypoint of the Longcut logic.
pub fn run_longcut(ctx: &Context) {
    run_inactive_mode(ctx)
}
