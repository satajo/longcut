use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

/// Runs the command given as the first shell argument as a background job and exits at once.
///
/// The job is reparented away from this process when the wrapper exits. A shell ignores SIGINT
/// and SIGQUIT in a background job and the ignore would survive exec, so the job resets them
/// before replacing itself with a plain `sh -c` of the command, which also gives the command
/// the empty parameter list a foreground command has.
const BACKGROUND_LAUNCH_WRAPPER: &str = r#"{ trap - INT QUIT; exec sh -c -- "$1"; } &"#;

/// Forking the job takes microseconds. bash retries a failed fork with growing sleeps for fifteen
/// seconds, so the wait is bounded to keep a failing launch from holding the session.
const BACKGROUND_LAUNCH_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug)]
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
    #[must_use]
    pub fn new(default_timeout: Duration) -> Self {
        Self { default_timeout }
    }

    /// Starts the command and returns as soon as it is running on its own.
    ///
    /// The command is started as a background job of a short-lived shell. It outlives this
    /// process, which never has to reap it.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to start, or if the launch cannot be awaited.
    pub fn run_async(command_string: &str) -> Result<(), RunError> {
        let mut wrapper = prepare_command(BACKGROUND_LAUNCH_WRAPPER);
        wrapper.arg("sh").arg(command_string);

        // The command has nothing to say to this process, so no IO is connected.
        wrapper.stdin(Stdio::null());
        wrapper.stdout(Stdio::null());
        wrapper.stderr(Stdio::null());

        let Ok(mut wrapper) = wrapper.spawn() else {
            return Err(RunError::Startup);
        };

        match wrapper.wait_timeout(BACKGROUND_LAUNCH_TIMEOUT) {
            Ok(Some(status)) if status.success() => Ok(()),
            // The wrapper exits with a failure only when the job could not be forked.
            Ok(Some(_)) => Err(RunError::Startup),
            // A wrapper still alive is retrying a failed fork, so the job has not started.
            // Ending the wrapper reports the failure instead of leaving the user waiting.
            Ok(None) => match wrapper.kill().and_then(|()| wrapper.wait()) {
                Ok(_) => Err(RunError::Startup),
                Err(_) => Err(RunError::Unknown),
            },
            Err(_) => Err(RunError::Unknown),
        }
    }

    /// # Errors
    ///
    /// Returns an error if the command fails to start, times out, or exits with a non-zero status.
    pub fn run_sync(&self, command_string: &str) -> Result<String, RunError> {
        let mut command = prepare_command(command_string);

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
}

/// A shell that runs the program string. The option list is terminated before the program
/// string so that a program starting with a dash is not read as shell options.
fn prepare_command(program_string: &str) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg("--").arg(program_string);
    cmd
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
        Ok(1..) => Some(buffer),
        Ok(0) | Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::service::{RunError, ShellService};
    use std::time::Duration;

    #[test]
    fn sync_run_result_is_ok_on_success() {
        let shell = ShellService::new(Duration::from_secs(1));
        let result = shell.run_sync("echo 'Hello, world!'");
        assert!(result.is_ok());
    }

    #[test]
    fn sync_run_result_contains_command_output_on_success() {
        let shell = ShellService::new(Duration::from_secs(1));
        let output = shell.run_sync("echo 'Hello, world!'").unwrap();
        assert_eq!(output, "Hello, world!\n");
    }

    #[test]
    fn sync_run_treats_a_leading_dash_as_part_of_the_command() {
        let shell = ShellService::new(Duration::from_secs(1));
        let result = shell.run_sync("-n");
        assert!(matches!(result, Err(RunError::Runtime(_))), "{result:?}");
    }
}
