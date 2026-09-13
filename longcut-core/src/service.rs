use crate::config::Config;
use crate::logic::{Context, run_layer_navigation_mode, run_window_mode};
use crate::model::key::{Key, Symbol};
use crate::model::session::SessionMode;
use crate::port::view::ViewModel;
use crate::port::{Launcher, WindowManager, executor::Executor, input::Input, view::View};

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

    /// Runs one session in the given mode, then returns.
    fn run_session(&self, mode: SessionMode) {
        // Command execution retries are always confirmed with Return.
        let keys_retry = [Key::new(Symbol::RETURN)];
        let context = Context {
            executor: self.executor,
            input: self.input,
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

    /// Runs one session per launch, indefinitely. Each session ends when its token is dropped at
    /// the end of the iteration.
    pub fn run_forever(&self, launcher: &dyn Launcher) -> ! {
        loop {
            let session = launcher.wait_for_launch();
            self.run_session(session.mode());
        }
    }
}
