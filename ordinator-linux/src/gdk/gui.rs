use crate::gdk::component::column::Column;
use crate::gdk::component::rectangle::Rectangle;
use crate::gdk::component::row::Row;
use crate::gdk::component::text::Text;
use crate::gdk::component::{Component, Context};
use crate::gdk::config::Config;
use crate::gdk::view_model::{Action, LayerView, ViewModel};
use crate::gdk::window::Window;
use gdk::cairo;

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
        cairo_context.set_font_size(24.0);

        let context = Context::new(&cairo_context).with_font_size(24);
        let component = render_layer_view(&model);
        component.render(&context);
    }
}

//-----------------------------------------------------------------------------
// Ui Components
//-----------------------------------------------------------------------------

fn render_single_layer_name(name: String) -> impl Component {
    Rectangle::new(Text::new(name))
}

fn render_layer_stack(layer_stack: &[String]) -> impl Component {
    let children = layer_stack
        .iter()
        .map(|name| render_single_layer_name(name.clone()))
        .collect();
    Row::new(children).gap_size(16)
}

fn render_single_action(action: &Action) -> impl Component {
    Row::new(vec![
        Text::new(action.shortcut.clone()),
        Text::new(action.name.clone()),
    ])
    .gap_size(8)
}

fn render_actions(actions: &[Action]) -> impl Component {
    let children = actions.iter().map(render_single_action).collect();
    Row::new(children).gap_size(16)
}

fn render_layer_view(model: &LayerView) -> impl Component {
    let layer_stack = render_layer_stack(&model.stack);
    let actions = render_actions(&model.actions);

    let children: Vec<Box<dyn Component>> = vec![Box::new(layer_stack), Box::new(actions)];
    let content = Column::new(children).gap_size(32);
    Rectangle::new(content).pad(32)
}
