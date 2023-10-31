#[derive(Debug)]
pub enum Parameter {
    Character,
    Choose(Vec<String>),
    Text,
}

#[derive(Debug)]
pub enum ParameterValue {
    Character(char),
    Choice(String),
    Text(String),
}
