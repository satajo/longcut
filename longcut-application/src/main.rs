use clap::{Parser, Subcommand};
use longcut_config::{ConfigError, ConfigModule, Module};
use longcut_core::CoreModule;
use longcut_gui::{ErrorScreen, GuiModule, GuiService, Screen};
use longcut_gui_adapter_longcut_core::GuiView;
use longcut_shell::ShellModule;
use longcut_shell_adapter_longcut_core::ShellExecutor;
use longcut_x11::X11Module;
use longcut_x11_adapter_longcut_core::{X11Input, X11WindowManager};
use longcut_xcb::XcbModule;
use longcut_xcb_adapter_longcut_gui::XcbWindowManager;
use std::fmt::Display;
use std::fs::{File, TryLockError};
use std::path::PathBuf;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

/// Without a subcommand, waits for the configured activation keys and runs the navigation
/// sessions they start, until killed.
#[derive(Parser)]
struct Args {
    /// Configuration file to use. Overrides the default path ~/.config/longcut/longcut.yaml
    #[clap(short, long, global = true)]
    config_file: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Check the configuration file for errors and exit. Exit code is 1 if any errors are detected.
    CheckConfig,
}

fn main() {
    let args = Args::parse();

    match args.command {
        None => run_application(&args),
        Some(Command::CheckConfig) => check_config(&args),
    }
}

fn check_config(args: &Args) {
    /// Utility for checking config validity that exits on error.
    fn check_module_config<M: Module>(config: &ConfigModule) {
        use ConfigError::{DeserializationError, KeyNotFound};
        if let Err(err) = config.config_for_module::<M>() {
            let module_name = M::IDENTIFIER;

            let error_message = match err {
                KeyNotFound => {
                    format!("Missing configuration for module {module_name}")
                }
                DeserializationError(err) => {
                    format!("Invalid configuration for module {module_name}: {err}")
                }
            };

            exit_with_error(&error_message);
        }
    }

    use longcut_config::InitError::{FileNotFound, ParsingError};

    let Some(config_file) = resolve_config_file_location(args) else {
        exit_with_error("Could not resolve configuration file path!");
    };

    println!("Checking configuration file: {}\n", config_file.display());

    let config = match ConfigModule::new(&config_file) {
        Ok(module) => module,
        Err(err) => {
            let message = match err {
                FileNotFound => "Could not find configuration file!".into(),
                ParsingError(err) => format!("Failed to parse configuration file: {err}!"),
            };

            exit_with_error(&message);
        }
    };

    check_module_config::<GuiModule>(&config);
    check_module_config::<ShellModule>(&config);
    check_module_config::<CoreModule>(&config);

    println!("No errors detected.");
    exit(0)
}

fn run_application(args: &Args) {
    let Some(config_file) = resolve_config_file_location(args) else {
        exit_with_error("Could not resolve configuration file path!");
    };

    let config = unwrap_module(ConfigModule::new(config_file));

    // A second instance would compete with the first for the activation keys, so it ends here,
    // before anything is set up or shown.
    let _instance_lock = unwrap_init("instance lock", acquire_instance_lock());

    // The GUI comes up first so that every later failure is shown on screen: the process is
    // started by the session, and nobody is watching its stderr.
    let xcb = XcbModule::new();
    let xcb_gui_window_manager = XcbWindowManager::new(&xcb.xcb_service);
    let gui = unwrap_module(GuiModule::new(&config, &xcb_gui_window_manager));
    let startup = Startup {
        gui: &gui.gui_service,
    };

    let shell = startup.unwrap(ShellModule::IDENTIFIER, ShellModule::new(&config));
    let x11 = X11Module::new();
    let x11_input = X11Input::new(&x11.x11_handle);
    let x11_window_manager = X11WindowManager::new(&x11.x11_handle);
    let gui_view = GuiView::new(&gui.gui_service);
    let shell_executor = ShellExecutor::new(&shell.service);
    let core = startup.unwrap(
        CoreModule::IDENTIFIER,
        CoreModule::new(
            &config,
            &x11_input,
            &gui_view,
            &shell_executor,
            &x11_window_manager,
        ),
    );

    core.longcut_service.run_forever();
}

/// How long a startup error stays on screen before the process exits. The keyboard is not grabbed
/// at that point, so nothing could dismiss it earlier.
const STARTUP_ERROR_DISPLAY_TIME: Duration = Duration::from_secs(4);

/// Startup steps that run once the GUI exists. A failure is shown on screen as well as printed.
struct Startup<'a> {
    gui: &'a GuiService<'a>,
}

impl Startup<'_> {
    fn unwrap<T, E: Display>(&self, subject: &str, init_result: Result<T, E>) -> T {
        match init_result {
            Ok(value) => value,
            Err(error) => self.fail(subject, error),
        }
    }

    fn fail(&self, subject: &str, error: impl Display) -> ! {
        let error_message = format!("{subject} initialization failed.\n\nCause: {error}");
        eprintln!("Error: {error_message}");
        self.gui.display_screen(Screen::Error(ErrorScreen {
            actions: vec![],
            error_type: "Startup failed".to_string(),
            error_details: error_message,
        }));
        sleep(STARTUP_ERROR_DISPLAY_TIME);
        exit(1)
    }
}

/// Holds an exclusive lock for the process lifetime so that at most one longcut instance runs at
/// a time. The kernel releases the lock when the descriptor closes, on any form of process death.
fn acquire_instance_lock() -> Result<File, String> {
    let Some(runtime_dir) = dirs::runtime_dir() else {
        return Err("XDG_RUNTIME_DIR is not set".to_string());
    };
    let path = runtime_dir.join("longcut.lock");
    let file = File::create(&path)
        .map_err(|error| format!("could not open {}: {error}", path.display()))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => {
            Err("another longcut instance is already running".to_string())
        }
        Err(TryLockError::Error(error)) => {
            Err(format!("could not lock {}: {error}", path.display()))
        }
    }
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
fn unwrap_module<M: Module, E: Display>(module_init_result: Result<M, E>) -> M {
    unwrap_init(M::IDENTIFIER, module_init_result)
}

/// Unwraps an initialization Result, logging and stopping the program on error.
fn unwrap_init<T, E: Display>(subject: &str, init_result: Result<T, E>) -> T {
    match init_result {
        Ok(value) => value,
        Err(error) => {
            let error_message = format!("{subject} initialization failed.\n\nCause: {error}");

            exit_with_error(&error_message);
        }
    }
}

/// Prints out the provided error message and termintaes the process.
fn exit_with_error(error_message: &str) -> ! {
    eprintln!("Error: {error_message}");
    exit(1)
}
