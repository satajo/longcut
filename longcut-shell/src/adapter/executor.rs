use crate::{RunError, ShellModule};
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
                RunError::StartupError => ExecutorError::StartupError,
                RunError::RuntimeError(details) => ExecutorError::RuntimeError(details),
                RunError::UnknownError => ExecutorError::UnknownError,
                RunError::TimeoutError => {
                    let message = "Execution timed out and was aborted".to_string();
                    ExecutorError::RuntimeError(message)
                }
            }),
        }
    }
}
