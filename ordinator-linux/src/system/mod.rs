use ordinator_core::model::command::Instruction;
use ordinator_core::port::executor::{Executor, ExecutorError};
use std::io::Read;
use std::process::{Child, Command, Stdio};

pub struct ShellExecutor;

impl ShellExecutor {
    pub fn new() -> Self {
        Self
    }

    fn prepare_command(instruction: &Instruction) -> Command {
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd.arg(&instruction.program_string);

        // "&" postfix orders the shell to evaluate a command in the background in a separate process.
        if !instruction.is_synchronous {
            cmd.arg("&");
        }

        cmd
    }

    // Spawn a command, capturing its stderr output.
    fn spawn_command(mut command: Command) -> Result<Child, ExecutorError> {
        command
            .stdout(Stdio::null())
            // Stderr is captured for synchronous instruction error handling.
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| ExecutorError::StartupError)
    }

    fn read_stderr_output(process: &mut Child) -> Result<String, ExecutorError> {
        let mut buffer = String::new();
        let mut stderr = process.stderr.take().unwrap();
        stderr
            .read_to_string(&mut buffer)
            .map_err(|_| ExecutorError::UnknownError)?;

        Ok(buffer.trim().to_string())
    }
}

impl Executor for ShellExecutor {
    fn execute(&self, instruction: &Instruction) -> Result<(), ExecutorError> {
        let command = Self::prepare_command(instruction);
        let mut process = Self::spawn_command(command)?;

        // Asynchronous instruction are let run in the background; all error checking is ignored.
        if !instruction.is_synchronous {
            return Ok(());
        }

        // Synchronous processes are waited to finish. If no errors occur, the execution is considered successful.
        let exit_status = process.wait().map_err(|_| ExecutorError::UnknownError)?;
        if exit_status.success() {
            return Ok(());
        }

        // Process exited with an error code. Let's try to extract the stderr and report that as a RuntimeError.
        let error_details = Self::read_stderr_output(&mut process)?;
        if !error_details.is_empty() {
            return Err(ExecutorError::RuntimeError(error_details));
        }

        // Extracting human-readable data from the error was not possible, so we're left with an UnknownError.
        Err(ExecutorError::UnknownError)
    }
}
