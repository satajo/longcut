use std::fmt::Debug;

// ----------------------------------------------------------------------------
// Variants for Parameter definitions and values.
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub enum ParameterDefinitionVariant {
    Character(CharacterParameter),
    Choose(ChooseParameter),
    Text(TextParameter),
}

#[derive(Debug)]
pub enum ParameterValueVariant {
    Character(ParameterValue<CharacterParameter>),
    Choose(ParameterValue<ChooseParameter>),
    Text(ParameterValue<TextParameter>),
}

// ----------------------------------------------------------------------------
// Parameter definitions
// ----------------------------------------------------------------------------

pub trait Parameter: Sized {
    type Value: Sized;

    /// Binds a value to the parameter. If the parameter-internal validation does not accept the
    /// value, an error is returned.
    fn try_assign_value(
        &self,
        value: impl Into<Self::Value>,
    ) -> Result<ParameterValue<Self>, &'static str>;
}

/// A value which has been assigned to a [Parameter].
///
/// A ParameterValue can only be built using the [Parameter::try_assign_value] method, providing
/// guarantees that the value is in fact compatible with the parameter.
#[derive(Debug)]
pub struct ParameterValue<P: Parameter>(P::Value);

impl<P: Parameter> ParameterValue<P> {
    pub fn take(self) -> P::Value {
        self.0
    }
}

/// A single character.
#[derive(Debug)]
pub struct CharacterParameter;

impl Parameter for CharacterParameter {
    type Value = char;

    fn try_assign_value(
        &self,
        value: impl Into<Self::Value>,
    ) -> Result<ParameterValue<Self>, &'static str> {
        Ok(ParameterValue(value.into()))
    }
}

/// A piece of text, a string.
#[derive(Debug)]
pub struct TextParameter;

impl Parameter for TextParameter {
    type Value = String;

    fn try_assign_value(
        &self,
        value: impl Into<Self::Value>,
    ) -> Result<ParameterValue<Self>, &'static str> {
        Ok(ParameterValue(value.into()))
    }
}

/// A list of pre-defined options to choose from.
#[derive(Debug)]
pub struct ChooseParameter {
    pub options: Vec<String>,
}

impl Parameter for ChooseParameter {
    type Value = String;

    fn try_assign_value(
        &self,
        value: impl Into<Self::Value>,
    ) -> Result<ParameterValue<Self>, &'static str> {
        let into_value = value.into();

        if !self.options.contains(&into_value) {
            return Err("provided value is not a valid option");
        }

        Ok(ParameterValue(into_value))
    }
}
