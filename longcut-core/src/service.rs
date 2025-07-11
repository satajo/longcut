use crate::config::Config;
use crate::logic::{Context, run_inactive_mode};
use crate::model::key::{Key, Symbol};
use crate::port::{executor::Executor, input::Input, view::View};

pub struct CoreService<'a> {
    executor: &'a dyn Executor,
    input: &'a dyn Input,
    view: &'a dyn View,
    config: Config,
}

impl<'a> CoreService<'a> {
    pub fn new(
        executor: &'a dyn Executor,
        input: &'a dyn Input,
        view: &'a dyn View,
        config: Config,
    ) -> Self {
        Self {
            executor,
            input,
            view,
            config,
        }
    }

    pub fn run_forever(&self) {
        let keys_retry = [Key::new(Symbol::Return)];
        let context = Context {
            executor: self.executor,
            input: self.input,
            view: self.view,
            keys_activate: &self.config.keys_activate,
            keys_back: &self.config.keys_back,
            keys_deactivate: &self.config.keys_deactivate,
            keys_retry: &keys_retry,
            root_layer: &self.config.root_layer,
        };

        loop {
            run_inactive_mode(&context);
        }
    }
}
