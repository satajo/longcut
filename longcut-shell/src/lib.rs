use crate::config::Config;
use longcut_config::{ConfigError, ConfigModule, Module};
use std::io::Read;
use std::process::{Child, Command, Stdio};
use wait_timeout::ChildExt;

pub mod adapter;
pub mod config;

pub struct ShellModule {
    config: Config,
}

impl Module for ShellModule {
    const IDENTIFIER: &'static str = "shell";

    type Config = Config;
}

pub enum RunError {
    StartupError,
    RuntimeError(String),
    TimeoutError,
    UnknownError,
}

impl ShellModule {
    pub fn new(config_module: &ConfigModule) -> Result<Self, ConfigError> {
        let config = config_module.config_for_module::<Self>()?;
        Ok(Self { config })
    }

    pub fn run_async(&self, command_string: &str) -> Result<(), RunError> {
        let mut command = self.prepare_command(command_string);

        // No IO is piped because we only care about starting the command.
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());

        match command.spawn() {
            Ok(_) => Ok(()),
            Err(_) => Err(RunError::StartupError),
        }
    }

    pub fn run_sync(&self, command_string: &str) -> Result<(), RunError> {
        let mut command = self.prepare_command(command_string);

        // Stdout and error streams are captured for error reporting.
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Process is spawned...
        let Ok(mut process) = command.spawn() else {
            return Err(RunError::StartupError);
        };

        // ...and awaited to finish within the specified timeout.
        let exit_status = match process.wait_timeout(self.config.default_timeout) {
            Ok(Some(status)) => status,
            // When wait returns without a status code, the process timed out and was aborted.
            Ok(None) => Err(RunError::TimeoutError)?,
            // Failing to wait for the process = ???
            Err(_) => Err(RunError::UnknownError)?,
        };

        // If exit status reports success, the execution is considered successful.
        if exit_status.success() {
            return Ok(());
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

        Err(RunError::RuntimeError(error_details))
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
