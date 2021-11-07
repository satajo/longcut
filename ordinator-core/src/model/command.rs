#[derive(Debug)]
pub struct Step {
    program: String,
}

impl Step {
    pub fn new(program: String) -> Self {
        Self { program }
    }
}

#[derive(Debug)]
pub struct Command {
    pub name: String,
    steps: Vec<Step>,
}

impl Command {
    pub fn new(name: String, steps: Vec<Step>) -> Self {
        Command { name, steps }
    }
}
