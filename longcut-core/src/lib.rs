//! Domain logic of longcut: layer navigation, command execution and parameter input, with every external capability entering through the port traits.

pub mod config;
mod logic;
pub mod model;
mod module;
pub mod port;
mod service;

pub use model::session::SessionMode;
pub use module::CoreModule;
pub use service::CoreService;
