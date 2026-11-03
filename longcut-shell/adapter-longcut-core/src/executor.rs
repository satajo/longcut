use longcut_core::port::executor::{Executor, ExecutorError};
use longcut_shell::{Poll, RunError, ShellService};
use std::time::{Duration, Instant};

/// How often a command run to completion is checked for exit.
const COMPLETION_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub struct ShellExecutor<'a> {
    shell: &'a ShellService,
}

impl<'a> ShellExecutor<'a> {
    #[must_use]
    pub fn new(shell: &'a ShellService) -> Self {
        Self { shell }
    }
}

impl Executor for ShellExecutor<'_> {
    /// Runs the command until it finishes or the configured timeout passes, at which point the
    /// command is killed and the timeout reported as the failure.
    fn run_to_completion(&self, program: &str) -> Result<String, ExecutorError> {
        let mut running = ShellService::start(program).map_err(into_executor_error)?;
        let started = Instant::now();
        loop {
            running = match running.poll() {
                Poll::Finished(result) => return result.map_err(into_executor_error),
                Poll::Running(running) => running,
            };
            if started.elapsed() >= self.shell.default_timeout() {
                return match running.kill() {
                    Ok(()) => Err(ExecutorError::Runtime(
                        "Execution timed out and was terminated".to_string(),
                    )),
                    Err(error) => Err(into_executor_error(error)),
                };
            }
            std::thread::sleep(COMPLETION_POLL_INTERVAL);
        }
    }

    fn run_in_background(&self, program: &str) -> Result<(), ExecutorError> {
        ShellService::run_async(program).map_err(into_executor_error)
    }
}

fn into_executor_error(error: RunError) -> ExecutorError {
    match error {
        RunError::Startup => ExecutorError::Startup,
        RunError::Runtime(details) => ExecutorError::Runtime(details),
        RunError::Unknown => ExecutorError::Unknown,
    }
}
