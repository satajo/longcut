/// Modes represent the different states that the program can be in.
///
/// The modes operate independently, handing over control to other modes when required. The modes
/// form a tree structure, being aware of the modes which they depend upon, but not of the modes
/// which depend on them.
mod command_execution;
mod error;
mod layer_navigation;
mod parameter_input;
mod window;

use crate::config::ApplicationConfig;
use crate::model::key::Key;
use crate::model::layer::Layer;
use crate::port::WindowManager;
use crate::port::executor::Executor;
use crate::port::input::Keyboard;
use crate::port::view::View;

pub(crate) use layer_navigation::run_layer_navigation_mode;
pub(crate) use window::run_window_mode;

/// Context is the container for the shared configuration and dependencies of the mode logic.
pub(crate) struct Context<'a> {
    pub executor: &'a dyn Executor,
    pub keyboard: &'a dyn Keyboard,
    pub view: &'a dyn View,
    pub window_manager: &'a dyn WindowManager,

    // Configuration
    pub keys_back: &'a [Key],
    pub keys_exit: &'a [Key],
    pub keys_retry: &'a [Key],

    // Layer
    pub root_layer: &'a Layer,
    pub app_specific_layers: &'a [ApplicationConfig],
}
