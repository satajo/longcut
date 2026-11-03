use nix::sys::memfd::{MFdFlags, memfd_create};
use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use std::fs::File;
use std::io::{Read, Seek};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

/// Starts shell commands.
#[derive(Debug)]
pub struct ShellService;

#[derive(Debug)]
pub enum RunError {
    Startup,
    Runtime(String),
    Unknown,
}

/// A command that has been started and not yet seen to finish.
///
/// The command's stdout and stderr are collected into in-memory files the kernel holds, so the
/// command never blocks on a full pipe while nobody reads, and the output is read once the
/// command has exited. The command and everything it spawns form one process group, so killing
/// the command kills all of it.
#[derive(Debug)]
pub struct RunningCommand {
    child: Child,
    stdout: File,
    stderr: File,
}

/// What a running command was found doing when it was polled.
#[derive(Debug)]
pub enum Poll {
    Running(RunningCommand),
    Finished(Result<String, RunError>),
}

impl ShellService {
    /// Starts the command and returns as soon as it is running on its own.
    ///
    /// The command is started as a background job of a short-lived shell. It outlives this
    /// process, which never has to reap it.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to start, or if the launch cannot be awaited.
    pub fn run_async(command_string: &str) -> Result<(), RunError> {
        /// Runs the command given as the first shell argument as a background job and exits at
        /// once.
        ///
        /// The job is reparented away from this process when the wrapper exits. A shell ignores
        /// SIGINT and SIGQUIT in a background job and the ignore would survive exec, so the job
        /// resets them before replacing itself with a plain `sh -c` of the command, which also
        /// gives the command the empty parameter list a foreground command has.
        const BACKGROUND_LAUNCH_WRAPPER: &str = r#"{ trap - INT QUIT; exec sh -c -- "$1"; } &"#;

        /// Forking the job takes microseconds. bash retries a failed fork with growing sleeps for
        /// fifteen seconds, so the wait is bounded to keep a failing launch from holding the
        /// session.
        const BACKGROUND_LAUNCH_TIMEOUT: Duration = Duration::from_secs(1);

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

    /// Starts the command and hands it back to be polled for its outcome or killed.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to start.
    pub fn start(command_string: &str) -> Result<RunningCommand, RunError> {
        let stdout = output_buffer("longcut-stdout")?;
        let stderr = output_buffer("longcut-stderr")?;

        let mut command = prepare_command(command_string);
        command.stdin(Stdio::null());
        command.stdout(Stdio::from(
            stdout.try_clone().map_err(|_error| RunError::Startup)?,
        ));
        command.stderr(Stdio::from(
            stderr.try_clone().map_err(|_error| RunError::Startup)?,
        ));
        // A group of its own, led by the shell, so that a kill reaches every process the
        // command spawned.
        command.process_group(0);

        let Ok(child) = command.spawn() else {
            return Err(RunError::Startup);
        };

        Ok(RunningCommand {
            child,
            stdout,
            stderr,
        })
    }
}

impl RunningCommand {
    /// Checks whether the command has finished, without waiting for it.
    #[must_use]
    pub fn poll(mut self) -> Poll {
        match self.child.try_wait() {
            Ok(None) => Poll::Running(self),
            Ok(Some(status)) => Poll::Finished(self.outcome(status)),
            Err(_) => Poll::Finished(Err(RunError::Unknown)),
        }
    }

    /// Ends the command and every process it spawned, and waits until the command is gone.
    ///
    /// The group is asked to terminate first and killed outright if any of it is still there
    /// after the grace period. A command that has already exited is simply reaped.
    ///
    /// # Errors
    ///
    /// Returns an error if the group cannot be signalled or the command cannot be waited for.
    pub fn kill(mut self) -> Result<(), RunError> {
        /// How long a killed command gets to act on SIGTERM before SIGKILL ends it.
        const TERMINATION_GRACE: Duration = Duration::from_secs(1);

        /// How often a killed command is checked for exit while the grace period runs.
        const TERMINATION_POLL_INTERVAL: Duration = Duration::from_millis(10);

        let group = self.process_group()?;

        signal_group(group, Signal::SIGTERM)?;
        let asked_to_terminate = Instant::now();
        let mut leader_reaped = false;
        while asked_to_terminate.elapsed() < TERMINATION_GRACE {
            if !leader_reaped {
                leader_reaped = match self.child.try_wait() {
                    Ok(status) => status.is_some(),
                    Err(_) => break,
                };
            }
            if leader_reaped && !group_exists(group)? {
                return Ok(());
            }
            std::thread::sleep(TERMINATION_POLL_INTERVAL);
        }

        signal_group(group, Signal::SIGKILL)?;
        if leader_reaped {
            return Ok(());
        }
        match self.child.wait() {
            Ok(_) => Ok(()),
            Err(_) => Err(RunError::Unknown),
        }
    }

    /// The command's process group, which the shell leads under its own pid.
    fn process_group(&self) -> Result<Pid, RunError> {
        let pid = i32::try_from(self.child.id()).map_err(|_error| RunError::Unknown)?;
        Ok(Pid::from_raw(pid))
    }

    fn outcome(&mut self, exit_status: ExitStatus) -> Result<String, RunError> {
        // If exit status reports success, the execution is considered successful.
        if exit_status.success() {
            return Ok(read_output(&mut self.stdout).unwrap_or_default());
        }

        // Process exited with an error code.
        let error_details = if let Some(stderr) = read_output(&mut self.stderr) {
            // Stderr printout is the preferred error message.
            stderr
        } else if let Some(stdout) = read_output(&mut self.stdout) {
            // Nothing usable was output to stderr, let's try stdout instead.
            stdout
        } else {
            // The command produced no output. Best we can do is the status code itself.
            exit_status.to_string()
        };

        Err(RunError::Runtime(error_details))
    }
}

/// Signals every process in the group. A group that no longer exists has nothing left to
/// signal, which is the wanted end state.
fn signal_group(group: Pid, signal: Signal) -> Result<(), RunError> {
    match killpg(group, signal) {
        Ok(()) | Err(nix::errno::Errno::ESRCH) => Ok(()),
        Err(_) => Err(RunError::Unknown),
    }
}

/// Whether any process, exited or not, is still in the group. The group's id stays reserved
/// while the group has a member, so the check cannot hit an unrelated process once the leader
/// has been reaped.
fn group_exists(group: Pid) -> Result<bool, RunError> {
    match killpg(group, None) {
        Ok(()) => Ok(true),
        Err(nix::errno::Errno::ESRCH) => Ok(false),
        Err(_) => Err(RunError::Unknown),
    }
}

/// A shell that runs the program string. The option list is terminated before the program
/// string so that a program starting with a dash is not read as shell options.
fn prepare_command(program_string: &str) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg("--").arg(program_string);
    cmd
}

/// An anonymous in-memory file for a command's output.
fn output_buffer(name: &str) -> Result<File, RunError> {
    match memfd_create(name, MFdFlags::MFD_CLOEXEC) {
        Ok(fd) => Ok(File::from(fd)),
        Err(_) => Err(RunError::Startup),
    }
}

/// The whole content of an output file, or nothing if it is empty or unreadable.
fn read_output(output: &mut File) -> Option<String> {
    output.rewind().ok()?;
    let mut buffer = String::new();
    match output.read_to_string(&mut buffer) {
        Ok(1..) => Some(buffer),
        Ok(0) | Err(_) => None,
    }
}
