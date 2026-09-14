//! Adapter implementing the core `Launcher`, `Input` and `WindowManager` ports with X11.

pub mod config;
mod input;
mod launcher;
mod window_manager;

pub use input::X11Input;
pub use launcher::{LauncherError, X11Launcher};
pub use window_manager::X11WindowManager;
