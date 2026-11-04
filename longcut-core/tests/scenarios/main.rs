//! Scenarios of the crate: each one serves a launch on a `CoreService` made of fake ports and
//! checks, through those ports, everything the session did. They reach the crate through its
//! public API only.

mod application_layers;
mod entering_parameters;
mod harness;
mod helpers;
mod navigating_layers;
mod running_commands;
mod sessions;
