//! Subprocess execution through `sh -c` with a configurable timeout.

pub mod config;
mod module;
mod service;

pub use module::ShellModule;
pub use service::{RunError, ShellService};
