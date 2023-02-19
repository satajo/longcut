use crate::module::{RunError, ShellModule};
use longcut_core::model::command::Instruction;
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
    fn execute(&self, instruction: &Instruction) -> Result<(), ExecutorError> {
        let program = &instruction.program_string;
        let result = if instruction.is_synchronous {
            self.shell.run_sync(program)
        } else {
            self.shell.run_async(program)
        };

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(match e {
                RunError::Startup => ExecutorError::StartupError,
                RunError::Runtime(details) => ExecutorError::RuntimeError(details),
                RunError::Unknown => ExecutorError::UnknownError,
                RunError::Timeout => {
                    let message = "Execution timed out and was aborted".to_string();
                    ExecutorError::RuntimeError(message)
                }
            }),
        }
    }
}
