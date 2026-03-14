use regex::Regex;
use std::collections::BTreeSet;

/// A concrete effect to be carried out. This is the rendered (parameter-substituted)
/// form of an effect template.
#[derive(Debug)]
pub enum Effect {
    ShellCommand {
        program: String,
        is_synchronous: bool,
    },
}

#[derive(Debug)]
enum Token {
    Text(String),
    Parameter(usize),
}

#[derive(Debug)]
pub struct ShellCommandTemplate {
    tokens: Vec<Token>,
    pub is_synchronous: bool,
}

#[derive(Debug)]
pub enum ShellCommandRenderError {
    MissingParameter,
}

impl ShellCommandTemplate {
    /// # Errors
    ///
    /// Returns an error if the program string is empty or contains invalid parameter placeholders.
    ///
    /// # Panics
    ///
    /// Panics if the regex pattern for parameter placeholders fails to compile.
    pub fn new(program: &str) -> Result<Self, String> {
        if program.is_empty() {
            return Err("program must not be an empty string".into());
        }

        // Program string is tokenized into a list.
        let pattern = Regex::new(r"\{([^{}]*)}").unwrap();

        let mut tokens: Vec<Token> = Vec::new();
        let mut last_match_end: usize = 0;
        for capture in pattern.captures_iter(program) {
            let full_match = capture.get(0).unwrap();

            // Capturing the command between each substitution.
            let slice = &program[last_match_end..full_match.start()];
            if !slice.is_empty() {
                tokens.push(Token::Text(slice.to_string()));
            }

            // Inserting the actual parameter substitution.
            let idx_str = capture.get(1).unwrap().as_str();
            let idx = idx_str
                .parse()
                .map_err(|_| format!("{idx_str} is not a valid parameter index"))?;
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

    /// Positive value indicates that the program executor should wait for this program to
    /// successfully exit before continuing on with the next program.
    pub fn set_synchronous(&mut self, value: bool) -> &mut Self {
        self.is_synchronous = value;
        self
    }

    /// # Errors
    ///
    /// Returns an error if a required parameter is missing.
    pub fn render(
        &self,
        parameters: &[impl AsRef<str>],
    ) -> Result<Effect, ShellCommandRenderError> {
        let mut program = String::new();
        for token in &self.tokens {
            match token {
                Token::Text(str) => {
                    program.push_str(str);
                }
                Token::Parameter(idx) => {
                    let value = parameters
                        .get(*idx)
                        .ok_or(ShellCommandRenderError::MissingParameter)?;

                    program.push_str(value.as_ref());
                }
            }
        }

        Ok(Effect::ShellCommand {
            program,
            is_synchronous: self.is_synchronous,
        })
    }

    #[must_use]
    pub fn get_required_parameters(&self) -> BTreeSet<usize> {
        let mut indexes = BTreeSet::new();
        for token in &self.tokens {
            if let Token::Parameter(idx) = token {
                indexes.insert(*idx);
            }
        }
        indexes
    }
}

/// A template for an effect that may contain parameter placeholders.
#[derive(Debug)]
pub enum EffectTemplate {
    ShellCommand(ShellCommandTemplate),
}

impl EffectTemplate {
    /// # Errors
    ///
    /// Returns an error if a required parameter is missing from the provided list.
    pub fn render(
        &self,
        parameters: &[impl AsRef<str>],
    ) -> Result<Effect, ShellCommandRenderError> {
        match self {
            EffectTemplate::ShellCommand(t) => t.render(parameters),
        }
    }

    #[must_use]
    pub fn get_required_parameters(&self) -> BTreeSet<usize> {
        match self {
            EffectTemplate::ShellCommand(t) => t.get_required_parameters(),
        }
    }

    pub fn set_synchronous(&mut self, value: bool) {
        match self {
            EffectTemplate::ShellCommand(t) => {
                t.set_synchronous(value);
            }
        }
    }
}

#[cfg(test)]
mod shell_effect_template_tests {
    use super::*;

    #[test]
    fn empty_string_is_not_allowed() {
        let empty_program = "";
        assert!(ShellCommandTemplate::new(empty_program).is_err());
    }

    #[test]
    fn empty_parameter_placeholder_is_not_allowed() {
        let program_with_empty_param = "echo {}";
        assert!(ShellCommandTemplate::new(program_with_empty_param).is_err());
    }

    #[test]
    fn parameterless_usage() {
        let program = "echo Hello!";
        let template = ShellCommandTemplate::new(program).unwrap();
        assert_eq!(template.get_required_parameters().len(), 0);

        let no_parameters: Vec<String> = vec![];
        let effect = template.render(&no_parameters).unwrap();
        let Effect::ShellCommand {
            program: rendered,
            is_synchronous,
        } = effect;
        assert_eq!(rendered, program);
        assert!(!is_synchronous);
    }

    #[test]
    fn single_parameter_usage() {
        let program_with_parameters = "echo 'Hello {2}'";
        let template = ShellCommandTemplate::new(program_with_parameters).unwrap();
        assert_eq!(template.get_required_parameters(), BTreeSet::from([2]));

        let parameters = ["foo", "bar", "baz"];
        let effect = template.render(&parameters).unwrap();
        let Effect::ShellCommand { program, .. } = effect;
        assert_eq!(program, "echo 'Hello baz'");
    }

    #[test]
    fn multiple_parameter_usage() {
        let program_with_parameters = "echo 'Hello {2}, {0}, and {2} again!'";
        let template = ShellCommandTemplate::new(program_with_parameters).unwrap();
        assert_eq!(template.get_required_parameters(), BTreeSet::from([0, 2]));

        let parameters = ["foo", "bar", "baz"];
        let effect = template.render(&parameters).unwrap();
        let Effect::ShellCommand { program, .. } = effect;
        assert_eq!(program, "echo 'Hello baz, foo, and baz again!'")
    }

    #[test]
    fn render_fails_when_parameters_are_missing() {
        let template_string = "echo 'Hello {0}!";
        let template = ShellCommandTemplate::new(template_string).unwrap();
        let no_parameters: Vec<String> = vec![];
        let result = template.render(&no_parameters);
        assert!(result.is_err());
    }
}
