pub mod action;
pub mod shortcut;

use ordinator_gui::component::row::Row;
use ordinator_gui::component::text::Text;
use ordinator_gui::model::color::Color;
use ordinator_gui::property::Property;
use ordinator_gui::Component;

pub fn view_root(
    background: Color,
    foreground: Color,
    border: Color,
    child: impl Component,
) -> impl Component {
    child
        .margin(20)
        .background(background)
        .border(1, border)
        .foreground(foreground)
}

pub fn render_layer_stack(layer_stack: &[String]) -> impl Component {
    let mut row = Row::new();
    for layer in layer_stack {
        row = row.add_child(Text::new(layer.clone()));
    }
    row.gap_size(20)
}
