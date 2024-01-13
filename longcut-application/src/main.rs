use adapter_longcut_gui_longcut_core::GuiView;
use clap::Parser;
use longcut_config::ConfigModule;
use longcut_core::CoreModule;
use longcut_gdk::GdkModule;
use longcut_gdk_adapter_longcut_gui::GdkWindowManager;
use longcut_gui::GuiModule;
use longcut_shell::ShellModule;
use longcut_shell_adapter_longcut_core::ShellExecutor;
use longcut_x11::X11Module;
use longcut_x11_adapter_longcut_core::X11Input;
use std::fmt::Debug;
use std::path::PathBuf;
use std::process::exit;

#[derive(Parser)]
struct Args {
    /// Configuration file to use. Overrides the default path ~/.config/longcut/longcut.yaml
    #[clap(short, long)]
    config_file: Option<String>,
}

fn main() {
    let args = Args::parse();
    let config_file: PathBuf = if let Some(path) = &args.config_file {
        PathBuf::from(path)
    } else if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push("longcut/longcut.yaml");
        config_dir
    } else {
        panic!("Could not resolve configuration file path!");
    };

    let config = ConfigModule::new(config_file).unwrap_or_else(|e| {
        exit_with_error("ConfigModule initialization failed!", e);
    });

    let x11 = X11Module::new();

    let gdk = GdkModule::new();

    let shell = ShellModule::new(&config)
        .unwrap_or_else(|e| exit_with_error("ShellModule initialization failed!", e));

    let gdk_gui_window_manager = GdkWindowManager::new(&gdk.gdk_service);
    let gui = GuiModule::new(&config, &gdk_gui_window_manager)
        .unwrap_or_else(|e| exit_with_error("GuiModule initialization failed!", e));

    let x11_input = X11Input::new(&x11.x11_handle);
    let gui_view = GuiView::new(&gui.gui_service);
    let shell_executor = ShellExecutor::new(&shell.service);
    let core = CoreModule::new(&config, &x11_input, &gui_view, &shell_executor)
        .unwrap_or_else(|e| exit_with_error("ConfigModule initialization failed!", e));

    core.longcut_service.run_forever();
}

/// Terminates the process and prints out the provided error message.
fn exit_with_error(description: impl AsRef<str>, cause: impl Debug) -> ! {
    eprintln!("Error: {}\nCause: {:?}", description.as_ref(), cause);
    exit(1)
}
