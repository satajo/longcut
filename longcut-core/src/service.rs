use crate::config::Config;
use crate::logic::{Context, run_layer_navigation_mode, run_window_mode};
use crate::model::key::{Key, Symbol};
use crate::model::session::SessionMode;
use crate::port::input::{InputError, Keyboard};
use crate::port::view::{ErrorViewModel, ViewModel};
use crate::port::{Launcher, WindowManager, executor::Executor, input::Input, view::View};
use std::thread::sleep;
use std::time::Duration;

/// How long the reason a session could not start stays on screen. Nothing can dismiss it
/// earlier, since the keyboard is exactly what could not be had.
const KEYBOARD_UNAVAILABLE_DISPLAY_TIME: Duration = Duration::from_secs(4);

pub struct CoreService<'a> {
    executor: &'a dyn Executor,
    input: &'a dyn Input,
    view: &'a dyn View,
    window_manager: &'a dyn WindowManager,
    config: Config,
}

impl<'a> CoreService<'a> {
    pub fn new(
        executor: &'a dyn Executor,
        input: &'a dyn Input,
        view: &'a dyn View,
        window_manager: &'a dyn WindowManager,
        config: Config,
    ) -> Self {
        Self {
            executor,
            input,
            view,
            window_manager,
            config,
        }
    }

    /// Runs one session in the given mode on the taken keyboard, then returns.
    fn run_session(&self, mode: SessionMode, keyboard: &dyn Keyboard) {
        // Command execution retries are always confirmed with Return.
        let keys_retry = [Key::new(Symbol::RETURN)];
        let context = Context {
            executor: self.executor,
            keyboard,
            view: self.view,
            window_manager: self.window_manager,
            keys_back: &self.config.keys_back,
            keys_exit: &self.config.keys_exit,
            keys_retry: &keys_retry,
            root_layer: &self.config.root_layer,
            app_specific_layers: &self.config.app_specific_layers,
        };
        match mode {
            SessionMode::Global => run_layer_navigation_mode(&context),
            SessionMode::Window => run_window_mode(&context),
        }
        self.view.render(ViewModel::None);
    }

    /// Runs one session per launch, indefinitely. The keyboard is taken for the whole of a
    /// session and given back when the session ends.
    pub fn run_forever(&self, launcher: &dyn Launcher) -> ! {
        loop {
            let mode = launcher.wait_for_launch();
            match self.input.take_keyboard() {
                Ok(keyboard) => self.run_session(mode, keyboard.as_ref()),
                Err(error) => self.show_keyboard_unavailable(&error),
            }
        }
    }

    /// Shows why the session could not start, then clears the screen and returns.
    fn show_keyboard_unavailable(&self, error: &InputError) {
        self.view.render(ViewModel::Error(ErrorViewModel {
            error_type: "Keyboard unavailable",
            error_details: &error.to_string(),
            actions: &[],
        }));
        sleep(KEYBOARD_UNAVAILABLE_DISPLAY_TIME);
        self.view.render(ViewModel::None);
    }
}

impl std::fmt::Debug for CoreService<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoreService")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}
