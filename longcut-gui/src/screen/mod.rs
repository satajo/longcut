use crate::component::row::Row;
use crate::component::text::Text;
use crate::component::Component;

pub mod error;
pub mod layer_navigation;
pub mod parameter_input;

pub fn render_layer_stack(layer_stack: &[String]) -> impl Component {
    let mut row = Row::new();
    for layer in layer_stack {
        row = row.add_child(Text::new(layer.clone()));
    }
    row.gap_size(20)
}
