#[derive(Debug)]
pub struct Step {
    pub program: String,
    pub is_synchronous: bool,
}

impl Step {
    pub fn new(program: String, is_synchronous: bool) -> Self {
        Self {
            program,
            is_synchronous,
        }
    }
}

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub steps: Vec<Step>,
    pub is_final: bool,
}

impl Command {
    pub fn new(name: String, steps: Vec<Step>, is_final: bool) -> Self {
        Command {
            name,
            steps,
            is_final,
        }
    }
}
