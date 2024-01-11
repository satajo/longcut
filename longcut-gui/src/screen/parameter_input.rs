use crate::component::action::Action;
use crate::component::layer_stack::LayerStack;
use crate::component::root::Root;
use crate::model::theme::Theme;
use longcut_core::port::view::{ParameterInputViewModel, ParameterVariant, ViewAction};
use longcut_graphics_lib::component::column::Column;
use longcut_graphics_lib::component::row::Row;
use longcut_graphics_lib::component::table::Table;
use longcut_graphics_lib::component::text::Text;
use longcut_graphics_lib::component::Component;
use longcut_graphics_lib::model::unit::Unit;
use longcut_graphics_lib::property::Property;

#[derive(Debug)]
pub struct ParameterInputScreen {
    pub parameter_name: String,
    pub stack: Vec<String>,
    variant: Variant,
}

#[derive(Debug)]
enum Variant {
    Character,
    String { current_input: String },
    Choose { options: Vec<Action> },
}

impl ParameterInputScreen {
    pub fn assemble(&self, theme: &Theme) -> Box<dyn Component> {
        let layer_stack = LayerStack::new(&self.stack).assemble();
        let content: Box<dyn Component> = match &self.variant {
            Variant::Character => {
                let prompt = Text::new(format!("{}:", self.parameter_name));

                let placeholder_text = Text::new("Any character".to_string());
                let placeholder_color = theme.placeholder_color.clone();
                let placeholder = placeholder_text.foreground(placeholder_color);

                Box::new(
                    Row::<Box<dyn Component>>::new()
                        .add_child(Box::new(prompt))
                        .add_child(Box::new(placeholder))
                        .gap_size(Unit::Em(1.0)),
                )
            }
            Variant::String { current_input } => {
                let prompt = Text::new(format!("{}:", self.parameter_name));

                let text: Box<dyn Component> = if current_input.is_empty() {
                    let placeholder_text = Text::new("Text".to_string());
                    let placeholder_color = theme.placeholder_color.clone();
                    Box::new(placeholder_text.foreground(placeholder_color))
                } else {
                    let input_text = Text::new(current_input.clone());
                    Box::new(input_text)
                };

                Box::new(
                    Row::<Box<dyn Component>>::new()
                        .add_child(Box::new(prompt))
                        .add_child(text)
                        .gap_size(Unit::Em(1.0)),
                )
            }
            Variant::Choose { options } => {
                let prompt = Text::new(format!("{}:", self.parameter_name));

                let mut options_table = Table::new(400);
                for option in options {
                    options_table = options_table.add_child(option.assemble(theme));
                }

                Box::new(
                    Column::<Box<dyn Component>>::new()
                        .add_child(Box::new(prompt))
                        .add_child(Box::new(options_table))
                        .gap_size(Unit::Em(1.0)),
                )
            }
        };

        let column = Column::<Box<dyn Component>>::new()
            .add_child(Box::new(layer_stack))
            .add_child(content)
            .gap_size(Unit::Em(1.0));

        let root = Root::new(
            theme.background_color.clone(),
            theme.foreground_color.clone(),
            theme.font.clone(),
            theme.border_color.clone(),
            column,
        );

        Box::new(root)
    }
}

impl From<ParameterInputViewModel<'_>> for ParameterInputScreen {
    fn from(data: ParameterInputViewModel) -> Self {
        let mut stack: Vec<String> = data
            .layer_stack
            .iter()
            .map(|i| i.name.to_string())
            .collect();

        stack.push(data.command.name.clone());

        let variant: Variant = match &data.parameter {
            ParameterVariant::CharInput => Variant::Character,
            ParameterVariant::StringInput { input_value } => Variant::String {
                current_input: input_value.to_string(),
            },
            ParameterVariant::OptionInput { options } => {
                let actions = options
                    .iter()
                    .map(|(key, action)| Action::new(key, &ViewAction::Branch(action.to_string())))
                    .collect();
                Variant::Choose { options: actions }
            }
        };

        Self {
            parameter_name: data.parameter_name.to_string(),
            stack,
            variant,
        }
    }
}
