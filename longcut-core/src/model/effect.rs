use regex::Regex;
use std::collections::BTreeSet;
use std::sync::LazyLock;

/// One parameter placeholder in a program string: an index in braces, such as `{0}`.
static PLACEHOLDER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{[^{}]*}").expect("the placeholder pattern is a valid regular expression")
});

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
    pub fn new(program: &str) -> Result<Self, String> {
        if program.is_empty() {
            return Err("program must not be an empty string".into());
        }

        // The text pieces between the placeholders and the placeholders themselves alternate,
        // starting and ending with a text piece, which is empty where two placeholders touch or
        // one sits at either end of the program.
        let mut tokens: Vec<Token> = Vec::new();
        let mut placeholders = PLACEHOLDER.find_iter(program);
        for text in PLACEHOLDER.split(program) {
            if !text.is_empty() {
                tokens.push(Token::Text(text.to_string()));
            }

            // The match holds exactly one brace at each end, since the pattern admits no brace
            // between them.
            if let Some(placeholder) = placeholders.next() {
                let idx_str = placeholder
                    .as_str()
                    .trim_start_matches('{')
                    .trim_end_matches('}');
                let idx = idx_str.parse().map_err(|error| {
                    format!("{idx_str} is not a valid parameter index: {error}")
                })?;
                tokens.push(Token::Parameter(idx));
            }
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
    fn placeholders_at_either_end_and_back_to_back_render() {
        let template = ShellCommandTemplate::new("{0}{1} and {2}").unwrap();
        assert_eq!(
            template.get_required_parameters(),
            BTreeSet::from([0, 1, 2])
        );

        let effect = template.render(&["a", "b", "c"]).unwrap();
        let Effect::ShellCommand { program, .. } = effect;
        assert_eq!(program, "ab and c");
    }

    #[test]
    fn multiple_parameter_usage() {
        let program_with_parameters = "echo 'Hello {2}, {0}, and {2} again!'";
        let template = ShellCommandTemplate::new(program_with_parameters).unwrap();
        assert_eq!(template.get_required_parameters(), BTreeSet::from([0, 2]));

        let parameters = ["foo", "bar", "baz"];
        let effect = template.render(&parameters).unwrap();
        let Effect::ShellCommand { program, .. } = effect;
        assert_eq!(program, "echo 'Hello baz, foo, and baz again!'");
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
