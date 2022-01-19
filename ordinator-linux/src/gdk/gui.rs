use crate::gdk::config::Config;
use crate::gdk::renderer::CairoRenderer;
use crate::gdk::view_model::{
    Action, ActionType, LayerNavigationViewModel, ParameterInputViewModel, ViewModel,
};
use crate::gdk::window::Window;
use gdk::cairo;
use ordinator_gui::component::column::Column;
use ordinator_gui::component::row::Row;
use ordinator_gui::component::table::Table;
use ordinator_gui::component::text::Text;
use ordinator_gui::model::color::Color;
use ordinator_gui::model::dimensions::Dimensions;
use ordinator_gui::property::Property;
use ordinator_gui::{Component, Context};

pub struct Gui<'a> {
    config: &'a Config,
    window: &'a Window<'a>,
}

impl<'a> Gui<'a> {
    pub fn new(config: &'a Config, window: &'a Window) -> Self {
        Self { config, window }
    }

    pub fn update(&self, model: ViewModel) {
        if let ViewModel::Invisible = model {
            return self.window.hide();
        }

        self.window.show(|cairo| {
            let renderer = CairoRenderer::new(&cairo).with_font_size(20);
            let color = Color::rgb(0, 0, 0);
            let draw_area = Dimensions::new(
                self.config.window.size.horizontal,
                self.config.window.size.vertical,
            );
            let ctx = Context::new(&renderer, &color, draw_area);
            match &model {
                // This will not happen, but the needs handling.
                ViewModel::Invisible => {}
                ViewModel::LayerNavigation(model) => {
                    render_layer_view(model).render(&ctx);
                }
                ViewModel::ParameterInput(model) => render_parameter_input_view(model).render(&ctx),
            }
        });
    }
}

//-----------------------------------------------------------------------------
// Views
//-----------------------------------------------------------------------------

fn render_parameter_input_view(model: &ParameterInputViewModel) -> impl Component {
    let layer_stack = render_layer_stack(&model.stack);
    let input_value: Box<dyn Component> = if model.current_input.is_empty() {
        Box::new(
            Text::new(model.parameter_placeholder.clone()).foreground(Color::rgb(127, 127, 127)),
        )
    } else {
        Box::new(Text::new(model.current_input.clone()))
    };

    let prompt_text = Text::new(format!("{}:", model.parameter_name));
    let prompt_line = Row::<Box<dyn Component>>::new()
        .add_child(Box::new(prompt_text))
        .add_child(input_value)
        .gap_size(20);

    let column = Column::<Box<dyn Component>>::new()
        .add_child(Box::new(layer_stack))
        .add_child(Box::new(prompt_line))
        .gap_size(20);
    view_root(column)
}

fn render_layer_view(model: &LayerNavigationViewModel) -> impl Component {
    let layer_stack = render_layer_stack(&model.stack);
    let actions = render_actions(&model.actions);
    let column = Column::<Box<dyn Component>>::new()
        .add_child(Box::new(layer_stack))
        .add_child(Box::new(actions))
        .gap_size(20);
    view_root(column)
}

fn view_root(child: impl Component) -> impl Component {
    child
        .margin(20)
        .background(Color::rgb(38, 38, 38))
        .border(1, Color::rgb(77, 77, 77))
        .foreground(Color::rgb(229, 229, 229))
}

fn render_layer_stack(layer_stack: &[String]) -> impl Component {
    let mut row = Row::new();
    for layer in layer_stack {
        row = row.add_child(Text::new(layer.clone()));
    }
    row.gap_size(20)
}

fn render_actions(actions: &[Action]) -> impl Component {
    let mut table = Table::new(400);
    for action in actions {
        table = table.add_child(render_single_action(action));
    }
    table
}

fn render_single_action(action: &Action) -> impl Component {
    let action_shortcut = Text::new(action.shortcut.clone()).width(125);
    let action_name = match &action.kind {
        ActionType::Branch { layer } => Text::new(layer.clone()),
        ActionType::Execute { program } => Text::new(program.clone()),
        ActionType::Unbranch => Text::new("Unbranch".into()),
        ActionType::Deactivate => Text::new("Deactivate".into()),
    };

    let action_color = match &action.kind {
        ActionType::Branch { .. } => Color::rgb(238, 118, 0),
        ActionType::Execute { .. } => Color::rgb(229, 229, 229),
        ActionType::Unbranch => Color::rgb(127, 127, 127),
        ActionType::Deactivate => Color::rgb(127, 127, 127),
    };

    Row::<Box<dyn Component>>::new()
        .add_child(Box::new(action_shortcut))
        .add_child(Box::new(action_name))
        .gap_size(8)
        .foreground(action_color)
}
