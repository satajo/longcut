mod config;
mod gdk;
mod system;
mod x11;

use crate::gdk::GdkApplication;
use crate::system::ShellExecutor;
use crate::x11::X11;
use ordinator_core::run;

fn main() {
    let path = "ordinator.yaml".as_ref();
    let configuration = match config::read_config(path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{:?}", err);
            return;
        }
    };

    let input = X11::new();
    let view = GdkApplication::new();
    let executor = ShellExecutor::new();

    run(&input, &view, &executor, configuration);
}
