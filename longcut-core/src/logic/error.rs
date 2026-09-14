use super::Context;
use crate::port::executor::ExecutorError;
use crate::port::view::{ErrorViewModel, ViewAction, ViewModel};

pub(crate) enum ErrorResult {
    Abort,
    Cancel,
    Retry,
}

/// Both informs and provides options for continuing when an error is encountered.
pub(crate) fn run_error_mode(ctx: &Context, error: &ExecutorError) -> ErrorResult {
    render(ctx, error);
    loop {
        let press = ctx.keyboard.capture_any();
        if ctx.keys_exit.contains(&press) {
            return ErrorResult::Abort;
        } else if ctx.keys_back.contains(&press) {
            return ErrorResult::Cancel;
        } else if ctx.keys_retry.contains(&press) {
            return ErrorResult::Retry;
        }
    }
}

fn render(ctx: &Context, error: &ExecutorError) {
    let error_type = match error {
        ExecutorError::Runtime(_) => "Runtime error",
        ExecutorError::Startup => "Startup error",
        ExecutorError::Unknown => "Unknown error",
    };

    let error_details = match error {
        ExecutorError::Runtime(details) => details.trim(),
        ExecutorError::Startup => "Failed to start the target command",
        ExecutorError::Unknown => "No error details available",
    };

    let mut actions = vec![];

    for key in ctx.keys_back {
        actions.push((key, ViewAction::Unbranch));
    }

    for key in ctx.keys_exit {
        actions.push((key, ViewAction::Exit));
    }

    for key in ctx.keys_retry {
        actions.push((key, ViewAction::Retry));
    }

    ctx.view.render(ViewModel::Error(ErrorViewModel {
        error_type,
        error_details,
        actions: &actions,
    }));
}
