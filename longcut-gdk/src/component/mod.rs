pub mod action;
pub mod shortcut;

use longcut_gui::component::row::Row;
use longcut_gui::component::text::Text;
use longcut_gui::model::color::Color;
use longcut_gui::property::Property;
use longcut_gui::Component;

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
