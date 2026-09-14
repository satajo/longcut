//! Overlay window management through XCB and cairo.

mod module;
mod service;
mod visual;
mod window;

pub use module::XcbModule;
pub use service::{XcbError, XcbService};
pub use window::{Window, WindowGeometry};
