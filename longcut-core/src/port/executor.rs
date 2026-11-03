use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutorError {
    Runtime(String),
    Startup,
    Unknown,
}

/// What a running command was found doing when it was polled.
#[derive(Debug)]
pub enum CommandPoll {
    /// Still running. The command is handed back to be polled again or killed.
    Running(Box<dyn RunningCommand>),
    /// Finished, with its output or with the details of its failure.
    Finished(Result<String, ExecutorError>),
}

/// A command that has been started and not yet seen to finish.
///
/// Both operations consume the command, so a finished command cannot be polled or killed
/// again, and a killed command cannot be polled.
pub trait RunningCommand: Debug {
    /// Checks on the command without waiting for it.
    fn poll(self: Box<Self>) -> CommandPoll;

    /// Ends the command and everything it spawned, and waits until it is gone.
    ///
    /// # Errors
    ///
    /// Returns an error if the command cannot be ended, leaving it in an unknown state.
    fn kill(self: Box<Self>) -> Result<(), ExecutorError>;
}

/// Executes string shaped shell commands and reports back how the execution went.
pub trait Executor {
    /// Starts the command and hands it back to be polled for its outcome or killed.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to start.
    fn start(&self, command: &str) -> Result<Box<dyn RunningCommand>, ExecutorError>;

    /// Executes the specified command in the background, continuing on as soon as the program
    /// was launched. Does not block, but also does not report on any errors besides the launch
    /// related ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to start.
    fn run_in_background(&self, command: &str) -> Result<(), ExecutorError>;
}
