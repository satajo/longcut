use crate::model::command::EffectRenderError::ParameterMissing;
use crate::model::effect::{Effect, EffectTemplate};
use crate::model::parameter::{Parameter, ParameterDefinitionVariant, ParameterValueVariant};
use itertools::{EitherOrBoth, Itertools};

#[derive(Debug)]
pub struct CommandParameter {
    pub name: String,
    pub parameter: ParameterDefinitionVariant,
}

impl CommandParameter {
    pub fn new(name: String, parameter: ParameterDefinitionVariant) -> Self {
        Self { name, parameter }
    }
}

#[derive(Debug)]
pub struct Command {
    pub name: String,
    steps: Vec<EffectTemplate>,
    parameters: Vec<CommandParameter>,
    pub is_final: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CommandError {
    NoStepsProvided,
    MissingParameter(usize),
    UnusedParameter(usize),
}

#[derive(Debug)]
pub enum EffectRenderError {
    ParameterDefinitionAndValueMismatch,
    ParameterMissing,
}

impl Command {
    pub fn new(
        name: String,
        steps: Vec<EffectTemplate>,
        parameters: Vec<CommandParameter>,
    ) -> Result<Self, CommandError> {
        // Command without any steps makes no sense.
        if steps.is_empty() {
            return Err(CommandError::NoStepsProvided);
        }

        // Parameters used by every step are collected into a single set for sanity checking.
        let mut required_parameters = std::collections::BTreeSet::new();
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

    pub fn get_parameters(&self) -> &Vec<CommandParameter> {
        &self.parameters
    }

    pub fn set_final(&mut self, value: bool) -> &mut Self {
        self.is_final = value;
        self
    }

    /// Renders out the command into an [Effect] sequence.
    ///
    /// The provided parameter values must equal in order, in type, and in value compatibility the
    /// values expected by this command. If this condition doesn't hold, the command rendering will
    /// fail with an error.
    pub fn render_effects(
        &self,
        values: Vec<ParameterValueVariant>,
    ) -> Result<Vec<Effect>, EffectRenderError> {
        let substitutions = gather_parameter_substitutions(&self.parameters, values)?;
        let effects = render_effect_templates(&self.steps, substitutions);
        return Ok(effects);

        /// Generates substitution strings for all the provided parameter definition-value pairs.
        fn gather_parameter_substitutions(
            parameters: &[CommandParameter],
            values: Vec<ParameterValueVariant>,
        ) -> Result<Vec<String>, EffectRenderError> {
            let mut substitutions: Vec<String> = vec![];

            // Substitutions are collected into the vector by iterating over (definition, value) pairs.
            let param_iter = parameters.iter();
            let value_iter = values.into_iter();
            for pair in param_iter.zip_longest(value_iter) {
                let EitherOrBoth::Both(definition, value) = pair else {
                    return Err(ParameterMissing);
                };

                let substitution = format_substitution_string(&definition.parameter, value)?;
                substitutions.push(substitution);
            }

            Ok(substitutions)
        }

        /// Renders the list of templates using the provided substitution strings values.
        fn render_effect_templates(
            templates: &[EffectTemplate],
            substitutions: Vec<String>,
        ) -> Vec<Effect> {
            let mut effects: Vec<Effect> = vec![];

            for template in templates {
                let panic_msg = "Internal error in template rendering. Debug command parameter validation process.";
                let effect = template.render(&substitutions).expect(panic_msg);
                effects.push(effect);
            }

            effects
        }

        /// Returns the substitution string for a single parameter definition-value -pair if possible.
        fn format_substitution_string(
            parameter_definition: &ParameterDefinitionVariant,
            parameter_value: ParameterValueVariant,
        ) -> Result<String, EffectRenderError> {
            use ParameterDefinitionVariant as Def;
            use ParameterValueVariant as Val;

            // Provided values must match the definitions. Although the parameter values are
            // already checked for validity during assignment operation, that type guarantee does
            // not carry through to here. We therefore perform another assignment to make sure that
            // every value matches the definition.
            match (&parameter_definition, parameter_value) {
                // Character parameter
                (Def::Character(definition), Val::Character(value)) => {
                    let Ok(verified) = definition.try_assign_value(value.take()) else {
                        return Err(EffectRenderError::ParameterDefinitionAndValueMismatch);
                    };

                    Ok(verified.take().to_string())
                }

                // Choose parameter
                (Def::Choose(definition), Val::Choose(value)) => {
                    let Ok(verified) = definition.try_assign_value(value.take()) else {
                        return Err(EffectRenderError::ParameterDefinitionAndValueMismatch);
                    };

                    Ok(verified.take().to_string())
                }

                // Text parameter
                (Def::Text(definition), Val::Text(value)) => {
                    let Ok(verified) = definition.try_assign_value(value.take()) else {
                        return Err(EffectRenderError::ParameterDefinitionAndValueMismatch);
                    };

                    Ok(verified.take().to_string())
                }

                // Parameter mismatch.
                _ => Err(EffectRenderError::ParameterDefinitionAndValueMismatch),
            }
        }
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;
    use crate::model::effect::ShellCommandTemplate;
    use crate::model::parameter::TextParameter;

    #[test]
    fn can_build_parameterless_command() {
        let greeter = EffectTemplate::ShellCommand(
            ShellCommandTemplate::new("echo 'Hello world!'".into()).unwrap(),
        );
        let result = Command::new("Greet the world".into(), vec![greeter], vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn stepless_command_cannot_be_built() {
        let result = Command::new("Do nothing".into(), vec![], vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn can_build_parameterless_multi_step_command() {
        let greet_you = EffectTemplate::ShellCommand(
            ShellCommandTemplate::new("echo 'Hi there!'".into()).unwrap(),
        );
        let greet_me = EffectTemplate::ShellCommand(
            ShellCommandTemplate::new("echo 'Hello myself!'".into()).unwrap(),
        );
        let result = Command::new("Greet us".into(), vec![greet_you, greet_me], vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn can_build_command_with_parameters() {
        let greet_target = EffectTemplate::ShellCommand(
            ShellCommandTemplate::new("echo 'Hi {0}!'".into()).unwrap(),
        );
        let param_target = CommandParameter::new(
            "Example".into(),
            ParameterDefinitionVariant::Text(TextParameter),
        );
        let result = Command::new("Greet".into(), vec![greet_target], vec![param_target]);
        assert!(result.is_ok());
    }

    #[test]
    fn required_parameters_must_be_declared() {
        let greet_target = EffectTemplate::ShellCommand(
            ShellCommandTemplate::new("echo 'Hi {0}!'".into()).unwrap(),
        );
        let result = Command::new("Greet".into(), vec![greet_target], vec![]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CommandError::MissingParameter(0));
    }

    #[test]
    fn declared_parameters_must_be_required() {
        let greet_target = EffectTemplate::ShellCommand(
            ShellCommandTemplate::new("echo 'Hello!'".into()).unwrap(),
        );
        let param_target = CommandParameter::new(
            "Example".into(),
            ParameterDefinitionVariant::Text(TextParameter),
        );
        let result = Command::new("Greet".into(), vec![greet_target], vec![param_target]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CommandError::UnusedParameter(0));
    }

    #[test]
    fn command_effects_can_be_rendered() {
        let greet_target = EffectTemplate::ShellCommand(
            ShellCommandTemplate::new("echo 'Hello {0}'".into()).unwrap(),
        );
        let param_target = CommandParameter::new(
            "Example".into(),
            ParameterDefinitionVariant::Text(TextParameter),
        );
        let command = Command::new("Greet".into(), vec![greet_target], vec![param_target]).unwrap();
        let values = vec![ParameterValueVariant::Text(
            TextParameter.try_assign_value("World").unwrap(),
        )];
        let effects = command.render_effects(values).unwrap();
        assert_eq!(effects.len(), 1);
        let Effect::ShellCommand { program, .. } = &effects[0];
        assert_eq!(program, "echo 'Hello World'");
    }
}
