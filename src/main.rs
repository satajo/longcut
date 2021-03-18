mod core;
mod gtk;
mod mock;
mod x11;

use crate::gtk::GtkApplication;
use crate::mock::controller::MockController;
use crate::x11::X11;

fn main() {
    let controller = X11::new();
    let view = GtkApplication::new();
    core::main(controller, view);
}
