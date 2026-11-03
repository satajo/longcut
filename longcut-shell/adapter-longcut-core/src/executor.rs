use longcut_core::port::executor::{CommandPoll, Executor, ExecutorError, RunningCommand};
use longcut_shell::{Poll, RunError, ShellService};

#[derive(Debug, Default)]
pub struct ShellExecutor;

impl ShellExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Executor for ShellExecutor {
    fn start(&self, program: &str) -> Result<Box<dyn RunningCommand>, ExecutorError> {
        let running = ShellService::start(program).map_err(into_executor_error)?;
        Ok(Box::new(ShellRunningCommand(running)))
    }

    fn run_in_background(&self, program: &str) -> Result<(), ExecutorError> {
        ShellService::run_async(program).map_err(into_executor_error)
    }
}

#[derive(Debug)]
struct ShellRunningCommand(longcut_shell::RunningCommand);

impl RunningCommand for ShellRunningCommand {
    fn poll(self: Box<Self>) -> CommandPoll {
        let ShellRunningCommand(running) = *self;
        match running.poll() {
            Poll::Running(running) => CommandPoll::Running(Box::new(ShellRunningCommand(running))),
            Poll::Finished(result) => CommandPoll::Finished(result.map_err(into_executor_error)),
        }
    }

    fn kill(self: Box<Self>) -> Result<(), ExecutorError> {
        let ShellRunningCommand(running) = *self;
        running.kill().map_err(into_executor_error)
    }
}

fn into_executor_error(error: RunError) -> ExecutorError {
    match error {
        RunError::Startup => ExecutorError::Startup,
        RunError::Runtime(details) => ExecutorError::Runtime(details),
        RunError::Unknown => ExecutorError::Unknown,
    }
}
