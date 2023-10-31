use crate::component::layer_stack::LayerStack;
use crate::theme::Theme;
use longcut_core::model::parameter::Parameter;
use longcut_core::port::view::ParameterInputViewModel;
use longcut_graphics_lib::component::column::Column;
use longcut_graphics_lib::component::root::Root;
use longcut_graphics_lib::component::row::Row;
use longcut_graphics_lib::component::text::Text;
use longcut_graphics_lib::component::Component;
use longcut_graphics_lib::property::Property;

#[derive(Debug)]
pub struct ParameterInputScreen {
    pub current_input: String,
    pub parameter_name: String,
    pub parameter_placeholder: String,
    pub stack: Vec<String>,
}

impl ParameterInputScreen {
    pub fn assemble(&self, theme: &Theme) -> Box<dyn Component> {
        let layer_stack = LayerStack::new(&self.stack).assemble();

        let input_prompt = Text::new(format!("{}:", self.parameter_name));
        let input_value: Box<dyn Component> = if self.current_input.is_empty() {
            let value = self.parameter_placeholder.clone();
            let color = theme.placeholder_color.clone();
            Box::new(Text::new(value).foreground(color))
        } else {
            Box::new(Text::new(self.current_input.clone()))
        };

        let prompt_line = Row::<Box<dyn Component>>::new()
            .add_child(Box::new(input_prompt))
            .add_child(input_value)
            .gap_size(20);

        let column = Column::<Box<dyn Component>>::new()
            .add_child(Box::new(layer_stack))
            .add_child(Box::new(prompt_line))
            .gap_size(20);

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

        let parameter_placeholder = match &data.parameter.parameter {
            Parameter::Character => "Any character".to_string(),
            Parameter::Text => "Text".to_string(),
            Parameter::Choose(options) => options.join(", "),
        };

        Self {
            current_input: data.input_value.to_string(),
            parameter_name: data.parameter.name.clone(),
            parameter_placeholder,
            stack,
        }
    }
}
