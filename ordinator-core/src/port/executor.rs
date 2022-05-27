use crate::model::command::Instruction;

#[derive(Debug)]
pub enum ExecutorError {
    RuntimeError(String),
    StartupError,
    UnknownError,
}

pub trait Executor {
    fn execute(&self, instruction: &Instruction) -> Result<(), ExecutorError>;
}
