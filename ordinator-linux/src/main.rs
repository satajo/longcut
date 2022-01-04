mod config;
mod gdk;
mod system;
mod x11;

use crate::gdk::GdkApplication;
use crate::system::ShellExecutor;
use crate::x11::X11;
use clap::Parser;
use ordinator_core::run;
use std::path::Path;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    config: String,
}

fn main() {
    let args = Args::parse();

    let path = Path::new(&args.config);
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
