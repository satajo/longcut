use super::{Component, Context};
use crate::gdk::component::rectangle::Rectangle;
use crate::gdk::config::Dimensions;

pub struct Column<C: Component> {
    children: Vec<C>,
}

impl<C: Component> Column<C> {
    pub fn new(children: Vec<C>) -> Self {
        Self { children }
    }

    pub fn gap_size(self, amount: u32) -> Column<Rectangle<C>> {
        let padded_children = self
            .children
            .into_iter()
            .map(|c| Rectangle::new(c).pad_bottom(amount))
            .collect::<Vec<_>>();
        Column {
            children: padded_children,
        }
    }
}

impl<C: Component> Component for Column<C> {
    fn render(&self, context: &Context) {
        let mut offset_context = context.offset(Dimensions::default());
        for child in self.children.iter() {
            child.render(&offset_context);
            offset_context.offset.vertical += child.measure(context).vertical;
        }
    }

    fn measure(&self, context: &Context) -> Dimensions {
        let child_dimensions = self.children.iter().map(|c| c.measure(context));

        // Width of a column is the width of the widest child.
        let width = child_dimensions
            .clone()
            .map(|d| d.vertical)
            .max()
            .unwrap_or_default();

        // Height of a column is the total height of all children.
        let height = child_dimensions.map(|d| d.horizontal).sum();

        Dimensions {
            horizontal: width,
            vertical: height,
        }
    }
}
