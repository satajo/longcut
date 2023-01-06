use longcut_graphics_lib::component::row::Row;
use longcut_graphics_lib::component::text::Text;
use longcut_graphics_lib::component::Component;

pub struct LayerStack(Vec<String>);

impl LayerStack {
    pub fn new(layers: &[String]) -> Self {
        Self(layers.to_vec())
    }

    pub fn assemble(&self) -> impl Component {
        let mut row = Row::new();

        for layer in &self.0 {
            row = row.add_child(Text::new(layer.clone()));
        }

        row.gap_size(20)
    }
}
