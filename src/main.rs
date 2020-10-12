mod core;
mod gtk;
mod mock;

use crate::gtk::GtkApplication;
use crate::mock::controller::MockController;

fn main() {
    let controller = MockController::new();
    let view = GtkApplication::new();
    core::main(&controller, &view);
}
