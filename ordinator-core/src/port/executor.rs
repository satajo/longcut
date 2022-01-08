use crate::model::command::Instruction;

#[derive(Debug)]
pub enum Error {
    RuntimeError,
    StartupError,
    UnknownError,
}

pub trait Executor {
    fn execute(&self, instruction: &Instruction) -> Result<(), Error>;
}
