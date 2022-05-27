use crate::gdk::component;
use crate::gdk::config::Theme;
use ordinator_core::model::command::ParameterVariant;
use ordinator_core::port::view::ParameterInputViewModel;
use ordinator_gui::component::column::Column;
use ordinator_gui::component::row::Row;
use ordinator_gui::component::text::Text;
use ordinator_gui::property::Property;
use ordinator_gui::Component;

#[derive(Debug)]
pub struct ParameterInputScreen {
    pub current_input: String,
    pub parameter_name: String,
    pub parameter_placeholder: String,
    pub stack: Vec<String>,
}

impl ParameterInputScreen {
    pub fn assemble(&self, theme: &Theme) -> impl Component {
        let layer_stack = component::render_layer_stack(&self.stack);

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

        component::view_root(
            theme.background_color.clone(),
            theme.foreground_color.clone(),
            theme.border_color.clone(),
            column,
        )
    }
}

impl From<ParameterInputViewModel<'_>> for ParameterInputScreen {
    fn from(data: ParameterInputViewModel) -> Self {
        let mut stack: Vec<String> = data.layers.iter().map(|layer| layer.name.clone()).collect();
        stack.push(data.command.name.clone());

        let parameter_placeholder = match data.parameter.variant {
            ParameterVariant::Character => "Any character",
            ParameterVariant::Text => "Text",
        }
        .to_string();

        Self {
            current_input: data.input_value.to_string(),
            parameter_name: data.parameter.name.clone(),
            parameter_placeholder,
            stack,
        }
    }
}
