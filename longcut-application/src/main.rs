use clap::Parser;
use longcut_config as Config;
use longcut_core::CoreModule;
use longcut_gdk::adapter::gui_renderer::GuiRenderer;
use longcut_gdk::GdkModule;
use longcut_gui as gui;
use longcut_gui::adapter::view::GuiView;
use longcut_gui::GuiModule;
use longcut_shell::adapter::executor::ShellExecutor;
use longcut_shell::ShellModule;
use longcut_x11::adapter::input::X11Input;
use longcut_x11::X11Module;
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

    let x11 = X11Module::new();
    let x11_input = X11Input::new(&x11);

    let gdk = GdkModule::new();
    let gdk_gui_renderer = GuiRenderer::new(&gdk);

    let gui = GuiModule::new(&gdk_gui_renderer, gui::config::Config::default());
    let gui_view = GuiView::new(&gui);

    let shell = ShellModule::new();
    let shell_executor = ShellExecutor::new(&shell);

    let core = CoreModule::new(&x11_input, &gui_view, &shell_executor, configuration);
    core.run();
}
