#[derive(Debug)]
pub struct Step {
    pub program: String,
}

impl Step {
    pub fn new(program: String) -> Self {
        Self { program }
    }
}

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub steps: Vec<Step>,
    pub synchronous: bool,
}

impl Command {
    pub fn new(name: String, steps: Vec<Step>) -> Self {
        Command {
            name,
            steps,
            synchronous: true,
        }
    }
}
