use ordinator_core::model::command::Instruction;
use ordinator_core::port::executor::{Error, Executor};
use std::process::Command;

pub struct ShellExecutor;

impl ShellExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Executor for ShellExecutor {
    fn execute(&self, instruction: &Instruction) -> Result<(), Error> {
        println!("Executing: {:?}", instruction);

        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd.arg(&instruction.program_string);

        if !instruction.is_synchronous {
            // "&" postfix orders the shell to evaluate a command in the background in a separate process.
            cmd.arg("&");
        }

        let mut process = cmd.spawn().map_err(|_| Error::StartupError)?;
        if instruction.is_synchronous {
            process.wait().map_err(|_| Error::RuntimeError)?;
        }

        Ok(())
    }
}
