use ordinator_core::model::command::Command;
use ordinator_core::port::executor::{Error, Executor};
use std::process::Command as Cmd;

pub struct ShellExecutor;

impl ShellExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Executor for ShellExecutor {
    fn execute(&self, command: &Command) -> Result<(), Error> {
        println!("Executing: {:?}", command);
        for step in &command.steps {
            let mut cmd = Cmd::new("sh");
            cmd.arg("-c");
            cmd.arg(&step.program);

            if !step.is_synchronous {
                // "&" postfix orders the shell to evaluate a command in the background in a separate process.
                cmd.arg("&");
            }

            let mut process = cmd.spawn().map_err(|_| Error::StartupError)?;
            if step.is_synchronous {
                process.wait().map_err(|_| Error::RuntimeError)?;
            }
        }
        Ok(())
    }
}
