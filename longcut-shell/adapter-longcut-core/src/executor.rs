use longcut_core::port::executor::{Executor, ExecutorError};
use longcut_shell::{RunError, ShellService};

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
    fn run_to_completion(&self, program: &str) -> Result<String, ExecutorError> {
        self.shell.run_sync(program).map_err(into_executor_error)
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
        RunError::Timeout => {
            let message = "Execution timed out and was aborted".to_string();
            ExecutorError::Runtime(message)
        }
    }
}
