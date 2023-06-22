use itertools::Itertools;
use longcut_graphics_lib::component::row::Row;
use longcut_graphics_lib::component::text::Text;
use longcut_graphics_lib::component::Component;

pub struct LayerStack(Vec<String>);

impl LayerStack {
    pub fn new(layers: &[String]) -> Self {
        Self(layers.to_vec())
    }

    pub fn assemble(&self) -> impl Component {
        let names = self.0.iter().map(|layer| layer.as_str());
        let names_with_separators = Itertools::intersperse(names, ">").map(String::from);

        let mut row = Row::new();
        for item in names_with_separators {
            row = row.add_child(Text::new(item));
        }

        row.gap_size(10)
    }
}
