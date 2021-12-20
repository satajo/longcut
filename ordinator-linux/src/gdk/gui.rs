use crate::gdk::config::Config;
use crate::gdk::renderer::CairoRenderer;
use crate::gdk::view_model::{Action, LayerView, ViewModel};
use crate::gdk::window::Window;
use gdk::cairo;
use ordinator_gui::component::column::Column;
use ordinator_gui::component::row::Row;
use ordinator_gui::model::color::Color;
use ordinator_gui::model::dimensions::Dimensions;
use ordinator_gui::primitive::text::Text;
use ordinator_gui::property::background::Background;
use ordinator_gui::property::foreground::Foreground;
use ordinator_gui::property::margin::Margin;
use ordinator_gui::property::size::Size;
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
        match model {
            ViewModel::Layer(visible_model) => {
                self.window
                    .show(|cr| self.update_layer_view(&cr, visible_model));
            }
            ViewModel::Invisible => {
                self.window.hide();
            }
        }
    }

    fn update_layer_view(&self, cairo_context: &cairo::Context, model: LayerView) {
        self.config.color_fg.apply(cairo_context);
        let renderer = CairoRenderer::new(cairo_context).with_font_size(24);
        let color = Color::rgb(0, 0, 0);
        let draw_area = Dimensions::new(
            self.config.window.size.horizontal,
            self.config.window.size.vertical,
        );
        let context = Context::new(&renderer, &color, draw_area);
        let component = render_layer_view(&model);
        component.render(&context);
    }
}

//-----------------------------------------------------------------------------
// Views
//-----------------------------------------------------------------------------

fn render_layer_view(model: &LayerView) -> impl Component {
    let layer_stack = render_layer_stack(&model.stack);
    let actions = render_actions(&model.actions);
    Column::<Box<dyn Component>>::new()
        .add_child(Box::new(layer_stack))
        .add_child(Box::new(actions))
        .gap_size(32)
        .margin(32)
}

fn render_layer_stack(layer_stack: &[String]) -> impl Component {
    let mut row = Row::new();
    for layer in layer_stack {
        row = row.add_child(render_single_layer_name(layer.clone()));
    }
    row.gap_size(16)
}

fn render_single_layer_name(name: String) -> impl Component {
    Text::new(name)
        .background(Color::rgb(0, 255, 255))
        .foreground(Color::rgb(255, 0, 255))
}

fn render_actions(actions: &[Action]) -> impl Component {
    let mut row = Row::new();
    for action in actions {
        row = row.add_child(render_single_action(action));
    }
    row.gap_size(16)
}

fn render_single_action(action: &Action) -> impl Component {
    let action_shortcut = Text::new(action.shortcut.clone())
        .foreground(Color::rgb(255, 255, 0))
        .max_width(50);

    let action_name = Text::new(action.name.clone())
        .foreground(Color::rgb(255, 255, 255))
        .max_width(300);

    Row::<Box<dyn Component>>::new()
        .add_child(Box::new(action_shortcut))
        .add_child(Box::new(action_name))
        .gap_size(8)
}
