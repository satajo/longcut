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
        for step in &command.steps {
            println!("Executing: {:?}", step);
            let mut process = Cmd::new("sh")
                .arg("-c")
                .arg(&step.program)
                .spawn()
                .map_err(|_| Error::StartupError)?;

            if command.synchronous {
                process.wait().map_err(|_| Error::RuntimeError)?;
            }
        }
        Ok(())
    }
}
