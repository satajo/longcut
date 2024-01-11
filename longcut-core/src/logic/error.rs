use super::Context;
use crate::port::executor::ExecutorError;
use crate::port::view::{ErrorViewModel, ViewAction, ViewModel};

pub enum ErrorResult {
    Abort,
    Cancel,
    Retry,
}

/// Both informs and provides options for continuing when an error is encountered.
pub fn run_error_mode(ctx: &Context, error: &ExecutorError) -> ErrorResult {
    render(ctx, error);
    loop {
        let press = ctx.input.capture_any();
        if ctx.keys_deactivate.contains(&press) {
            return ErrorResult::Abort;
        } else if ctx.keys_back.contains(&press) {
            return ErrorResult::Cancel;
        } else if ctx.keys_retry.contains(&press) {
            return ErrorResult::Retry;
        }
    }
}

fn render(ctx: &Context, error: &ExecutorError) {
    let mut actions = vec![];

    for key in ctx.keys_back {
        actions.push((key, ViewAction::Unbranch));
    }

    for key in ctx.keys_deactivate {
        actions.push((key, ViewAction::Deactivate));
    }

    for key in ctx.keys_retry {
        actions.push((key, ViewAction::Retry));
    }

    ctx.view.render(ViewModel::Error(ErrorViewModel {
        error,
        actions: &actions,
    }));
}
