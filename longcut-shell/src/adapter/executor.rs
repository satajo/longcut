use crate::module::{RunError, ShellModule};
use longcut_core::port::executor::{Executor, ExecutorError};

pub struct ShellExecutor<'a> {
    shell: &'a ShellModule,
}

impl<'a> ShellExecutor<'a> {
    pub fn new(shell: &'a ShellModule) -> Self {
        Self { shell }
    }
}

impl<'a> Executor for ShellExecutor<'a> {
    fn run_to_completion(&self, program: &str) -> Result<(), ExecutorError> {
        self.shell.run_sync(program).map_err(|error| error.into())
    }

    fn run_in_background(&self, program: &str) -> Result<(), ExecutorError> {
        self.shell.run_async(program).map_err(|error| error.into())
    }
}

impl From<RunError> for ExecutorError {
    fn from(value: RunError) -> Self {
        match value {
            RunError::Startup => ExecutorError::StartupError,
            RunError::Runtime(details) => ExecutorError::RuntimeError(details),
            RunError::Unknown => ExecutorError::UnknownError,
            RunError::Timeout => {
                let message = "Execution timed out and was aborted".to_string();
                ExecutorError::RuntimeError(message)
            }
        }
    }
}
