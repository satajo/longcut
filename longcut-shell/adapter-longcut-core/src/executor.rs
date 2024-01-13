use longcut_core::port::executor::{Executor, ExecutorError};
use longcut_shell::{RunError, ShellService};

pub struct ShellExecutor<'a> {
    shell: &'a ShellService,
}

impl<'a> ShellExecutor<'a> {
    pub fn new(shell: &'a ShellService) -> Self {
        Self { shell }
    }
}

impl<'a> Executor for ShellExecutor<'a> {
    fn run_to_completion(&self, program: &str) -> Result<String, ExecutorError> {
        self.shell.run_sync(program).map_err(into_executor_error)
    }

    fn run_in_background(&self, program: &str) -> Result<(), ExecutorError> {
        self.shell.run_async(program).map_err(into_executor_error)
    }
}

fn into_executor_error(error: RunError) -> ExecutorError {
    match error {
        RunError::Startup => ExecutorError::StartupError,
        RunError::Runtime(details) => ExecutorError::RuntimeError(details),
        RunError::Unknown => ExecutorError::UnknownError,
        RunError::Timeout => {
            let message = "Execution timed out and was aborted".to_string();
            ExecutorError::RuntimeError(message)
        }
    }
}
