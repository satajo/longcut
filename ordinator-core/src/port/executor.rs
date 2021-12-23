use crate::model::command::Command;

#[derive(Debug)]
pub enum Error {
    RuntimeError,
    StartupError,
    UnknownError,
}

pub trait Executor {
    fn execute(&self, command: &Command) -> Result<(), Error>;
}
