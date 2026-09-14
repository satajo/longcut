//! The longcut executable: wires the modules and adapters together and runs the command line interface.

use clap::{Parser, Subcommand};
use longcut_config::{ConfigError, ConfigModule, Module};
use longcut_core::CoreModule;
use longcut_gui::{ErrorScreen, GuiModule, GuiService, Screen};
use longcut_gui_adapter_longcut_core::GuiView;
use longcut_shell::ShellModule;
use longcut_shell_adapter_longcut_core::ShellExecutor;
use longcut_x11::X11Module;
use longcut_x11_adapter_longcut_core::{X11Input, X11Launcher, X11WindowManager};
use longcut_xcb::XcbModule;
use longcut_xcb_adapter_longcut_gui::XcbWindowManager;
use std::fmt::Display;
use std::fs::{File, TryLockError};
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration;

/// Without a subcommand, binds the configured launch keys and runs one navigation session per
/// press, until killed.
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

fn main() -> ExitCode {
    let args = Args::parse();

    let result = match args.command {
        None => run_application(&args),
        Some(Command::CheckConfig) => check_config(&args),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error_message) => {
            eprintln!("Error: {error_message}");
            ExitCode::FAILURE
        }
    }
}

fn check_config(args: &Args) -> Result<(), String> {
    fn check_module_config<M: Module>(config: &ConfigModule) -> Result<(), String> {
        use ConfigError::{DeserializationError, KeyNotFound};
        let module_name = M::IDENTIFIER;

        match config.config_for_module::<M>() {
            Ok(_) => Ok(()),
            Err(KeyNotFound) => Err(format!("Missing configuration for module {module_name}")),
            Err(DeserializationError(err)) => Err(format!(
                "Invalid configuration for module {module_name}: {err}"
            )),
        }
    }

    use longcut_config::InitError::{ParsingError, ReadError};

    let config_file = resolve_config_file_location(args)?;

    println!("Checking configuration file: {}\n", config_file.display());

    let config = ConfigModule::new(&config_file).map_err(|err| match err {
        ReadError(err) => format!("Could not read configuration file: {err}!"),
        ParsingError(err) => format!("Failed to parse configuration file: {err}!"),
    })?;

    check_module_config::<GuiModule>(&config)?;
    check_module_config::<ShellModule>(&config)?;
    check_module_config::<CoreModule>(&config)?;
    check_module_config::<X11Launcher>(&config)?;

    println!("No errors detected.");
    Ok(())
}

fn run_application(args: &Args) -> Result<(), String> {
    let config_file = resolve_config_file_location(args)?;

    let config = ConfigModule::new(config_file)
        .map_err(|error| init_error(ConfigModule::IDENTIFIER, error))?;

    // A second instance would compete with the first for the launch keys, so it ends here,
    // before anything is set up or shown.
    let _instance_lock =
        acquire_instance_lock().map_err(|error| init_error("instance lock", error))?;

    // The GUI comes up first so that every later failure is shown on screen: the process is
    // spawned by the session, and nobody is watching its stderr.
    let xcb = XcbModule::new().map_err(|error| init_error(XcbModule::IDENTIFIER, error))?;
    let xcb_gui_window_manager = XcbWindowManager::new(&xcb.xcb_service);
    let gui = GuiModule::new(&config, &xcb_gui_window_manager)
        .map_err(|error| init_error(GuiModule::IDENTIFIER, error))?;
    let startup = Startup {
        gui: &gui.gui_service,
    };

    let shell = startup.check(ShellModule::IDENTIFIER, ShellModule::new(&config))?;
    let x11 = startup.check("x11", X11Module::new())?;
    let x11_input = X11Input::new(&x11.x11_handle);
    let x11_window_manager = X11WindowManager::new(&x11.x11_handle);
    let gui_view = GuiView::new(&gui.gui_service);
    let shell_executor = ShellExecutor::new(&shell.service);
    let core = startup.check(
        CoreModule::IDENTIFIER,
        CoreModule::new(
            &config,
            &x11_input,
            &gui_view,
            &shell_executor,
            &x11_window_manager,
        ),
    )?;

    // The launch keys are bound last, once every other section has parsed, so that a
    // configuration error never costs the user a keyboard grab.
    let launcher = startup.check(
        X11Launcher::IDENTIFIER,
        X11Launcher::new(&config, &x11.x11_handle),
    )?;
    core.longcut_service.run_forever(&launcher)
}

/// How long a startup error stays on screen before the process exits. The keyboard is not grabbed
/// at that point, so nothing could dismiss it earlier.
const STARTUP_ERROR_DISPLAY_TIME: Duration = Duration::from_secs(4);

/// Startup steps that run once the GUI exists. A failure is shown on screen before it is
/// reported.
struct Startup<'a> {
    gui: &'a GuiService<'a>,
}

impl Startup<'_> {
    fn check<T, E: Display>(&self, subject: &str, init_result: Result<T, E>) -> Result<T, String> {
        init_result.map_err(|error| {
            let error_message = init_error(subject, error);
            self.gui
                .display_screen(Screen::Error(ErrorScreen::without_actions(
                    "Startup failed".to_string(),
                    error_message.clone(),
                )));
            sleep(STARTUP_ERROR_DISPLAY_TIME);
            error_message
        })
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

fn resolve_config_file_location(args: &Args) -> Result<PathBuf, String> {
    // Config file provided as a command argument always takes priority.
    if let Some(path) = &args.config_file {
        return Ok(PathBuf::from(path));
    }

    // When no config file argument is passed, we try to read the file from the user's config directory.
    if let Some(mut config_dir_path) = dirs::config_dir() {
        config_dir_path.push("longcut/longcut.yaml");
        return Ok(config_dir_path);
    }

    // We don't know where to read the file from.
    Err("Could not resolve configuration file path!".to_string())
}

fn init_error(subject: &str, error: impl Display) -> String {
    format!("{subject} initialization failed.\n\nCause: {error}")
}
