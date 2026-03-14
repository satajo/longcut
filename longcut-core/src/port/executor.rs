#[derive(Debug)]
pub enum ExecutorError {
    RuntimeError(String),
    StartupError,
    UnknownError,
}

/// Executes string shaped shell commands and reports back how the execution went.
pub trait Executor {
    /// Executes the specified command synchronously, blocking until the execution finishes.
    ///
    /// On success, the command output is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to start or exits with an error.
    fn run_to_completion(&self, command: &str) -> Result<String, ExecutorError>;

    /// Executes the specified command in the background, continuing on as soon as the program
    /// was launched. Does not block, but also does not report on any errors besides the launch
    /// related ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to start.
    fn run_in_background(&self, command: &str) -> Result<(), ExecutorError>;
}
