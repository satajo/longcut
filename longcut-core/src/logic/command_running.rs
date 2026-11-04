use super::Context;
use crate::port::executor::{CommandPoll, ExecutorError, RunningCommand};
use crate::port::view::{CommandRunningViewModel, ViewAction, ViewModel};
use std::time::Duration;

/// How a watched command ended.
pub(crate) enum RunningResult {
    /// The command finished on its own, with its output or the details of its failure.
    Finished(Result<String, ExecutorError>),
    /// The user killed the command with a back key.
    Cancelled,
    /// The user killed the command with an exit key.
    Aborted,
    /// The user asked for the command to be killed, but it could not be ended and is in an
    /// unknown state; it may still be running.
    KillFailed(ExecutorError),
}

/// Watches a running command until it finishes or the user kills it.
///
/// The command is checked on between short waits for a key. Once it has outlived the delay, a
/// screen shows what runs and for how long, refreshed whenever the displayed second changes. A
/// back or exit key kills the command; any other key is dropped.
pub(crate) fn run_command_running_mode(
    ctx: &Context,
    title: &str,
    program: &str,
    mut running: Box<dyn RunningCommand>,
) -> RunningResult {
    /// How long a command runs unannounced before the running screen appears. A command that
    /// finishes within this time never shows a screen, so quick commands do not flash one.
    const RUNNING_SCREEN_DELAY: Duration = Duration::from_millis(200);

    /// How long each wait for a key lasts before the command is checked on again.
    const KEY_WAIT: Duration = Duration::from_millis(50);

    let started = ctx.clock.now();
    let mut shown_second = None;
    loop {
        running = match running.poll() {
            CommandPoll::Finished(result) => return RunningResult::Finished(result),
            CommandPoll::Running(running) => running,
        };

        let elapsed = ctx.clock.now().saturating_duration_since(started);
        if elapsed >= RUNNING_SCREEN_DELAY && shown_second != Some(elapsed.as_secs()) {
            render(ctx, title, program, elapsed);
            shown_second = Some(elapsed.as_secs());
        }

        if let Some(press) = ctx.keyboard.capture_within(KEY_WAIT) {
            if ctx.keys_back.contains(&press) {
                return kill(running, RunningResult::Cancelled);
            }
            if ctx.keys_exit.contains(&press) {
                return kill(running, RunningResult::Aborted);
            }
        }
    }
}

fn kill(running: Box<dyn RunningCommand>, outcome: RunningResult) -> RunningResult {
    match running.kill() {
        Ok(()) => outcome,
        Err(error) => RunningResult::KillFailed(error),
    }
}

fn render(ctx: &Context, title: &str, program: &str, elapsed: Duration) {
    let mut actions = vec![];
    for key in ctx.keys_back {
        actions.push((key, ViewAction::Kill));
    }
    for key in ctx.keys_exit {
        actions.push((key, ViewAction::Kill));
    }

    ctx.view
        .render(ViewModel::CommandRunning(CommandRunningViewModel {
            actions: &actions,
            command_name: title,
            program,
            elapsed,
        }));
}
