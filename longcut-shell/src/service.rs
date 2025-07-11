use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

pub struct ShellService {
    default_timeout: Duration,
}

#[derive(Debug)]
pub enum RunError {
    Startup,
    Runtime(String),
    Timeout,
    Unknown,
}

impl ShellService {
    pub fn new(default_timeout: Duration) -> Self {
        Self { default_timeout }
    }

    pub fn run_async(&self, command_string: &str) -> Result<(), RunError> {
        let mut command = self.prepare_command(command_string);

        // No IO is piped because we only care about starting the command.
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());

        match command.spawn() {
            Ok(_) => Ok(()),
            Err(_) => Err(RunError::Startup),
        }
    }

    pub fn run_sync(&self, command_string: &str) -> Result<String, RunError> {
        let mut command = self.prepare_command(command_string);

        // Stdout and error streams are captured for error reporting.
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Process is spawned...
        let Ok(mut process) = command.spawn() else {
            return Err(RunError::Startup);
        };

        // ...and awaited to finish within the specified timeout.
        let exit_status = match process.wait_timeout(self.default_timeout) {
            Ok(Some(status)) => status,
            // When wait returns without a status code, the process timed out and was aborted.
            Ok(None) => Err(RunError::Timeout)?,
            // Failing to wait for the process = ???
            Err(_) => Err(RunError::Unknown)?,
        };

        // If exit status reports success, the execution is considered successful.
        if exit_status.success() {
            let output = read_stdout_output(&mut process).unwrap_or_default();
            return Ok(output);
        }

        // Process exited with an error code.
        let error_details = if let Some(stderr) = read_stderr_output(&mut process) {
            // Stderr printout is the preferred error message.
            stderr
        } else if let Some(stdout) = read_stdout_output(&mut process) {
            // Nothing usable was output to stderr, let's try stdout instead.
            stdout
        } else {
            // The command produced no output. Best we can do is the status code itself.
            exit_status.to_string()
        };

        Err(RunError::Runtime(error_details))
    }

    fn prepare_command(&self, program_string: &str) -> Command {
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd.arg(program_string);
        cmd
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

#[cfg(test)]
mod tests {
    use crate::service::ShellService;
    use std::time::Duration;

    #[test]
    fn sync_run_result_is_ok_on_success() {
        let shell = ShellService::new(Duration::from_secs(1));
        let result = shell.run_sync("echo 'Hello, world!'");
        assert!(result.is_ok())
    }

    #[test]
    fn sync_run_result_contains_command_output_on_success() {
        let shell = ShellService::new(Duration::from_secs(1));
        let output = shell.run_sync("echo 'Hello, world!'").unwrap();
        assert_eq!(output, "Hello, world!\n")
    }
}
