use regex::Regex;
use std::collections::BTreeSet;

#[derive(Debug)]
enum Token {
    Text(String),
    Parameter(usize),
}

#[derive(Debug)]
pub struct Instruction {
    pub program_string: String,
    pub is_synchronous: bool,
}

#[derive(Debug)]
pub struct InstructionTemplate {
    tokens: Vec<Token>,
    pub is_synchronous: bool,
}

#[derive(Debug)]
pub enum TemplateRenderError {
    MissingParameter,
}

impl InstructionTemplate {
    pub fn new(program: String) -> Result<Self, String> {
        if program.is_empty() {
            return Err("program must not be an empty string".into());
        }

        // Program string is tokenized into a list.
        let pattern = Regex::new(r"\{([^{}]*)}").unwrap();

        let mut tokens: Vec<Token> = Vec::new();
        let mut last_match_end: usize = 0;
        for capture in pattern.captures_iter(&program) {
            let full_match = capture.get(0).unwrap();

            // Capturing the command between each substitution.
            let slice = &program[last_match_end..full_match.start()];
            if !slice.is_empty() {
                tokens.push(Token::Text(slice.to_string()))
            }

            // Inserting the actual parameter substitution.
            let idx_str = capture.get(1).unwrap().as_str();
            let idx = idx_str
                .parse()
                .map_err(|_| format!("{} is not a valid parameter index", idx_str))?;
            tokens.push(Token::Parameter(idx));

            last_match_end = full_match.end();
        }

        // The remainder of the program string is added as the final text token.
        let slice = &program[last_match_end..];
        if !slice.is_empty() {
            tokens.push(Token::Text(slice.to_string()));
        }

        Ok(Self {
            tokens,
            is_synchronous: false,
        })
    }

    /// Positive value indicates that the program executor should wait for this program to successfully
    /// exit before continuing on with the next program.
    pub fn set_synchronous(&mut self, value: bool) -> &mut Self {
        self.is_synchronous = value;
        self
    }

    /// Applies the provided parameters into the command placeholders.
    pub fn apply_parameters(&self, parameters: &[&str]) -> String {
        let mut program = String::new();
        for token in self.tokens.iter() {
            match token {
                Token::Text(str) => program.push_str(str),
                Token::Parameter(idx) => {
                    let value = parameters.get(*idx).unwrap();
                    program.push_str(value);
                }
            }
        }
        program
    }

    pub fn render(
        &self,
        parameters: &[impl AsRef<str>],
    ) -> Result<Instruction, TemplateRenderError> {
        let mut program_string = String::new();
        for token in self.tokens.iter() {
            match token {
                Token::Text(str) => program_string.push_str(str),
                Token::Parameter(idx) => {
                    let value = parameters
                        .get(*idx)
                        .ok_or(TemplateRenderError::MissingParameter)?;
                    program_string.push_str(value.as_ref());
                }
            }
        }

        Ok(Instruction {
            program_string,
            is_synchronous: self.is_synchronous,
        })
    }

    fn get_required_parameters(&self) -> BTreeSet<usize> {
        let mut indexes = BTreeSet::new();
        for token in self.tokens.iter() {
            if let Token::Parameter(idx) = token {
                indexes.insert(*idx);
            }
        }
        indexes
    }
}

#[derive(Debug)]
pub enum ParameterDeclaration {
    Character,
    Text,
}

#[derive(Debug)]
pub enum ParameterValue {
    Character(char),
    Text(String),
    Choose(usize),
}

#[derive(Debug)]
pub struct Command {
    pub name: String,
    steps: Vec<InstructionTemplate>,
    parameters: Vec<ParameterDeclaration>,
    pub is_final: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CommandError {
    NoStepsProvided,
    MissingParameter(usize),
    UnusedParameter(usize),
}

#[derive(Debug)]
pub enum CommandRenderError {
    ParameterDeclarationAndValueMismatch,
    ParameterMissing,
}

impl Command {
    pub fn new(
        name: String,
        steps: Vec<InstructionTemplate>,
        parameters: Vec<ParameterDeclaration>,
    ) -> Result<Self, CommandError> {
        // Command without any steps makes no sense.
        if steps.is_empty() {
            return Err(CommandError::NoStepsProvided);
        }

        // Parameters used by every step are collected into a single step for sanity checking.
        let mut required_parameters: BTreeSet<usize> = BTreeSet::new();
        for parameter in steps.iter().flat_map(|step| step.get_required_parameters()) {
            required_parameters.insert(parameter);
        }

        // Every required parameter must be declared.
        for idx in &required_parameters {
            if parameters.get(*idx).is_none() {
                return Err(CommandError::MissingParameter(*idx));
            }
        }

        // Every declared parameter must be required.
        for idx in 0..parameters.len() {
            if !required_parameters.contains(&idx) {
                return Err(CommandError::UnusedParameter(idx));
            }
        }

        Ok(Self {
            name,
            steps,
            parameters,
            is_final: false,
        })
    }

    pub fn get_parameters(&self) -> &Vec<ParameterDeclaration> {
        &self.parameters
    }

    pub fn set_final(&mut self, value: bool) -> &mut Self {
        self.is_final = value;
        self
    }

    pub fn render_instructions(
        &self,
        parameters: &[ParameterValue],
    ) -> Result<Vec<Instruction>, CommandRenderError> {
        let mut substitutions: Vec<String> = vec![];

        // Provided parameters must match the declaration.
        for (idx, declaration) in self.parameters.iter().enumerate() {
            let value = parameters
                .get(idx)
                .ok_or(CommandRenderError::ParameterMissing)?;
            match declaration {
                ParameterDeclaration::Character => {
                    if let ParameterValue::Character(c) = value {
                        substitutions.push(c.to_string());
                    } else {
                        return Err(CommandRenderError::ParameterDeclarationAndValueMismatch);
                    }
                }
                ParameterDeclaration::Text => {
                    if let ParameterValue::Text(text) = value {
                        substitutions.push(text.clone());
                    } else {
                        return Err(CommandRenderError::ParameterDeclarationAndValueMismatch);
                    }
                }
            }
        }

        // Instruction templates are rendered by applying the parameters.
        let mut instructions: Vec<Instruction> = vec![];
        for template in &self.steps {
            let instruction = template.render(&substitutions).expect(
                "Internal error in template rendering. Debug command parameter validation process.",
            );
            instructions.push(instruction);
        }

        Ok(instructions)
    }
}

#[cfg(test)]
mod instruction_tests {
    use super::*;

    #[test]
    fn empty_string_is_not_allowed() {
        let empty_program = "";
        assert!(InstructionTemplate::new(empty_program.into()).is_err());
    }

    #[test]
    fn empty_parameter_placeholder_is_not_allowed() {
        let program_with_empty_param = "echo {}";
        assert!(InstructionTemplate::new(program_with_empty_param.into()).is_err());
    }

    #[test]
    fn parameterless_usage() {
        let program = "echo Hello!";
        let template = InstructionTemplate::new(program.into()).unwrap();
        assert_eq!(template.get_required_parameters().len(), 0);

        let no_parameters: Vec<String> = vec![];
        let instruction = template.render(&no_parameters).unwrap();
        assert_eq!(instruction.program_string, program);
        assert_eq!(instruction.is_synchronous, false);
    }

    #[test]
    fn single_parameter_usage() {
        let program_with_parameters = "echo 'Hello {2}'";
        let template = InstructionTemplate::new(program_with_parameters.into()).unwrap();
        assert_eq!(template.get_required_parameters(), BTreeSet::from([2]));

        let parameters = ["foo", "bar", "baz"];
        let instruction = template.render(&parameters).unwrap();
        assert_eq!(instruction.program_string, "echo 'Hello baz'");
    }

    #[test]
    fn multiple_parameter_usage() {
        let program_with_parameters = "echo 'Hello {2}, {0}, and {2} again!'";
        let template = InstructionTemplate::new(program_with_parameters.into()).unwrap();
        assert_eq!(template.get_required_parameters(), BTreeSet::from([0, 2]));

        let parameters = ["foo", "bar", "baz"];
        let instruction = template.render(&parameters).unwrap();
        assert_eq!(
            instruction.program_string,
            "echo 'Hello baz, foo, and baz again!'"
        )
    }

    #[test]
    fn render_fails_when_parameters_are_missing() {
        let template_string = "echo 'Hello {0}!";
        let template = InstructionTemplate::new(template_string.into()).unwrap();
        let no_parameters: Vec<String> = vec![];
        let result = template.render(&no_parameters);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;

    #[test]
    fn can_build_parameterless_command() {
        let greeter = InstructionTemplate::new("echo 'Hello world!'".into()).unwrap();
        let result = Command::new("Greet the world".into(), vec![greeter], vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn instructionless_command_cannot_be_built() {
        let result = Command::new("Do nothing".into(), vec![], vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn can_build_parameterless_multi_step_command() {
        let greet_you = InstructionTemplate::new("echo 'Hi there!'".into()).unwrap();
        let greet_me = InstructionTemplate::new("echo 'Hello myself!'".into()).unwrap();
        let result = Command::new("Greet us".into(), vec![greet_you, greet_me], vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn can_build_command_with_parameters() {
        let greet_target = InstructionTemplate::new("echo 'Hi {0}!'".into()).unwrap();
        let param_target = ParameterDeclaration::Text;
        let result = Command::new("Greet".into(), vec![greet_target], vec![param_target]);
        assert!(result.is_ok());
    }

    #[test]
    fn required_parameters_must_be_declared() {
        let greet_target = InstructionTemplate::new("echo 'Hi {0}!'".into()).unwrap();
        let result = Command::new("Greet".into(), vec![greet_target], vec![]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CommandError::MissingParameter(0));
    }

    #[test]
    fn declared_parameters_must_be_required() {
        let greet_target = InstructionTemplate::new("echo 'Hello!'".into()).unwrap();
        let param_target = ParameterDeclaration::Text;
        let result = Command::new("Greet".into(), vec![greet_target], vec![param_target]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CommandError::UnusedParameter(0));
    }

    #[test]
    fn command_instructions_can_be_rendered() {
        let greet_target = InstructionTemplate::new("echo 'Hello {0}'".into()).unwrap();
        let param_target = ParameterDeclaration::Text;
        let command = Command::new("Greet".into(), vec![greet_target], vec![param_target]).unwrap();
        let values = vec![ParameterValue::Text("World".into())];
        let instructions = command.render_instructions(&values).unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0].program_string, "echo 'Hello World'");
    }
}
