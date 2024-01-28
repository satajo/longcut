use clap::Parser;
use longcut_config::{ConfigModule, Module};
use longcut_core::CoreModule;
use longcut_gdk::GdkModule;
use longcut_gdk_adapter_longcut_gui::GdkWindowManager;
use longcut_gui::GuiModule;
use longcut_gui_adapter_longcut_core::GuiView;
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
    let Some(config_file) = resolve_config_file_location(&args) else {
        exit_with_error("Could not resolve configuration file path!");
    };

    let config = unwrap_module(ConfigModule::new(config_file));

    let x11 = X11Module::new();

    let gdk = GdkModule::new();

    let shell = unwrap_module(ShellModule::new(&config));

    let gdk_gui_window_manager = GdkWindowManager::new(&gdk.gdk_service);
    let gui = unwrap_module(GuiModule::new(&config, &gdk_gui_window_manager));

    let x11_input = X11Input::new(&x11.x11_handle);
    let gui_view = GuiView::new(&gui.gui_service);
    let shell_executor = ShellExecutor::new(&shell.service);
    let core = unwrap_module(CoreModule::new(&config, &x11_input, &gui_view, &shell_executor));

    core.longcut_service.run_forever();
}

fn resolve_config_file_location(args: &Args) -> Option<PathBuf> {
    // Config file provided as a command argument always takes priority.
    if let Some(path) = &args.config_file {
        return Some(PathBuf::from(path));
    }

    // When no config file argument is passed, we try to read the file from the user's config directory.
    if let Some(mut config_dir_path) = dirs::config_dir() {
        config_dir_path.push("longcut/longcut.yaml");
        return Some(config_dir_path);
    }

    // We don't know where to read the file from.
    None
}

/// Unwraps a module-containing Result, logging and stopping the program on error.
fn unwrap_module<M: Module, E: Debug>(module_init_result: Result<M, E>) -> M {
    match module_init_result {
        Ok(module) => module,
        Err(error) => {
            let module_name = M::IDENTIFIER;
            let error_message = format!("{module_name} module initialization failed.\n\nCause: {error:?}");
            exit_with_error(&error_message);
        }
    }
}

/// Prints out the provided error message and termintaes the process.
fn exit_with_error(error_message: &str) -> ! {
    eprintln!("Error: {error_message}");
    exit(1)
}
