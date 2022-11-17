use clap::Parser;
use longcut_config as Config;
use longcut_core::run;
use longcut_gdk::GdkApplication;
use longcut_shell::ShellExecutor;
use longcut_x11::X11;
use std::path::Path;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    config: String,
}

fn main() {
    let args = Args::parse();

    let path = Path::new(&args.config);
    let configuration = match Config::read_config(path) {
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
