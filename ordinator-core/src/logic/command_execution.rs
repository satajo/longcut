use crate::model::command::Command;
use crate::port::executor::{Error, Executor};

pub struct CommandExecutionProgram<'a> {
    executor: &'a dyn Executor,
}

impl<'a> CommandExecutionProgram<'a> {
    pub fn new(executor: &'a impl Executor) -> Self {
        Self { executor }
    }

    pub fn run(&self, command: &Command) {
        match self.executor.execute(command) {
            Ok(_) => {}
            Err(error) => {
                println!("Execution failed! {:?}", error)
            }
        }
    }
}
