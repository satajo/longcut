use crate::model::key::Key;
use crate::port::executor::ExecutorError;
use crate::port::input::Input;
use crate::port::view::{ErrorViewModel, View, ViewAction, ViewModel};

pub struct ErrorProgram<'a> {
    input: &'a dyn Input,
    view: &'a dyn View,
    // Configuration
    keys_back: &'a [Key],
    keys_deactivate: &'a [Key],
    keys_retry: &'a [Key],
}

pub enum ProgramResult {
    Abort,
    Cancel,
    Retry,
}

impl<'a> ErrorProgram<'a> {
    pub fn new(
        input: &'a dyn Input,
        view: &'a dyn View,
        keys_back: &'a [Key],
        keys_deactivate: &'a [Key],
        keys_retry: &'a [Key],
    ) -> Self {
        Self {
            input,
            view,
            keys_back,
            keys_deactivate,
            keys_retry,
        }
    }

    pub fn run(&self, error: &ExecutorError) -> ProgramResult {
        self.render(error);
        loop {
            let press = self.input.capture_any();
            if self.keys_deactivate.contains(&press) {
                return ProgramResult::Abort;
            } else if self.keys_back.contains(&press) {
                return ProgramResult::Cancel;
            } else if self.keys_retry.contains(&press) {
                return ProgramResult::Retry;
            }
        }
    }

    fn render(&self, error: &ExecutorError) {
        let mut actions = vec![];

        for key in self.keys_back {
            actions.push((key, ViewAction::Unbranch));
        }

        for key in self.keys_deactivate {
            actions.push((key, ViewAction::Deactivate));
        }

        for key in self.keys_retry {
            actions.push((key, ViewAction::Retry));
        }

        self.view.render(ViewModel::Error(ErrorViewModel {
            error,
            actions: &actions,
        }));
    }
}
