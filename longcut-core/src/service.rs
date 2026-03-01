use crate::config::Config;
use crate::logic::{Context, run_inactive_mode};
use crate::model::key::{Key, Symbol};
use crate::port::{WindowManager, executor::Executor, input::Input, view::View};

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

    pub fn run_forever(&self) {
        let keys_retry = [Key::new(Symbol::Return)];
        let context = Context {
            executor: self.executor,
            input: self.input,
            view: self.view,
            window_manager: self.window_manager,
            keys_activate: &self.config.keys_activate,
            keys_app_activate: &self.config.keys_app_activate,
            keys_back: &self.config.keys_back,
            keys_deactivate: &self.config.keys_deactivate,
            keys_retry: &keys_retry,
            root_layer: &self.config.root_layer,
            application_shortcuts: self.config.application_shortcuts.as_ref(),
        };

        loop {
            run_inactive_mode(&context);
        }
    }
}
