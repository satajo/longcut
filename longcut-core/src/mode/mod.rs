/// Modes represent the different states that the program can be in.
///
/// The modes operate independently, handing over control to other modes when required. The modes
/// form a tree structure, being aware of the modes which they depend upon, but not of the modes
/// which depend on them.
///
/// The root of the mode tree starts at the [ParameterInputMode].
pub mod command_execution;
pub mod error;
pub mod inactive;
pub mod layer_navigation;
pub mod parameter_input;
