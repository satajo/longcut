#[derive(Debug)]
pub enum Parameter {
    Character,
    Text,
}

#[derive(Debug)]
pub enum ParameterValue {
    Character(char),
    Text(String),
}
