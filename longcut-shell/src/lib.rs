//! Subprocess execution through `sh -c`: synchronous commands as pollable, killable process
//! groups, background commands detached from this process.

mod service;

pub use service::{Poll, RunError, RunningCommand, ShellService};
