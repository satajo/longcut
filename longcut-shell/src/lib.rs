use longcut_core::model::command::Instruction;
use longcut_core::port::executor::{Executor, ExecutorError};
use std::io::Read;
use std::process::{Child, Command, Stdio};

pub struct ShellExecutor;

impl ShellExecutor {
    pub fn new() -> Self {
        Self
    }

    fn prepare_command(program_string: &str) -> Command {
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd.arg(program_string);
        cmd
    }
}

impl Executor for ShellExecutor {
    fn execute(&self, instruction: &Instruction) -> Result<(), ExecutorError> {
        let command = Self::prepare_command(&instruction.program_string);

        // Asynchronous instruction are let run in the background; all error checking is ignored.
        if !instruction.is_synchronous {
            AsyncExecutable(command).execute()
        } else {
            SyncExecutable(command).execute()
        }
    }
}

struct AsyncExecutable(Command);

impl AsyncExecutable {
    fn execute(mut self) -> Result<(), ExecutorError> {
        self.0.stdout(Stdio::null());
        self.0.stderr(Stdio::null());
        match self.0.spawn() {
            Ok(_) => Ok(()),
            Err(_) => Err(ExecutorError::StartupError),
        }
    }
}

struct SyncExecutable(Command);

impl SyncExecutable {
    fn execute(mut self) -> Result<(), ExecutorError> {
        // Stdout and error streams are captured for error reporting.
        self.0.stdout(Stdio::piped());
        self.0.stderr(Stdio::piped());

        // Process is spawned and awaited to finish with a status code.
        let mut process = self.0.spawn().map_err(|_| ExecutorError::StartupError)?;
        let exit_status = process.wait().map_err(|_| ExecutorError::UnknownError)?;

        // If exit status reports success, the execution is considered successful.
        if exit_status.success() {
            return Ok(());
        }

        // Process exited with an error code, let's use the stderr printout as error message.
        if let Some(error_details) = read_stderr_output(&mut process) {
            return Err(ExecutorError::RuntimeError(error_details));
        }

        // Nothing usable was output to stderr, let's try stdout instead.
        if let Some(error_details) = read_stdout_output(&mut process) {
            return Err(ExecutorError::RuntimeError(error_details));
        }

        // Extracting human-readable data from the error was not possible, best we can do is the
        // status code itself.
        Err(ExecutorError::RuntimeError(exit_status.to_string()))
    }
}

fn read_stderr_output(process: &mut Child) -> Option<String> {
    let stderr = process.stderr.take()?;
    read_stdio_buffer_into_string(stderr)
}

fn read_stdout_output(process: &mut Child) -> Option<String> {
    let stdout = process.stdout.take()?;
    read_stdio_buffer_into_string(stdout)
}

fn read_stdio_buffer_into_string(mut stream: impl Read) -> Option<String> {
    let mut buffer = String::new();
    match stream.read_to_string(&mut buffer) {
        Ok(0) => None,
        Ok(_) => Some(buffer),
        Err(_) => None,
    }
}
