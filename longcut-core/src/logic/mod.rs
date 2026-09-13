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
use crate::port::input::Input;
use crate::port::view::View;

pub use layer_navigation::run_layer_navigation_mode;
pub use window::run_window_mode;

/// Context is the container for the shared configuration and dependencies of the mode logic.
pub struct Context<'a> {
    pub executor: &'a dyn Executor,
    pub input: &'a dyn Input,
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
