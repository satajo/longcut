use crate::gdk::component::rectangle::Rectangle;
use crate::gdk::component::{Component, Context};
use crate::gdk::config::Dimensions;

pub struct Row<C: Component> {
    children: Vec<C>,
}

impl<C: Component> Row<C> {
    pub fn new(children: Vec<C>) -> Self {
        Self { children }
    }

    pub fn gap_size(self, amount: u32) -> Row<Rectangle<C>> {
        let padded_children = self
            .children
            .into_iter()
            .map(|c| Rectangle::new(c).pad_right(amount))
            .collect();
        Row {
            children: padded_children,
        }
    }
}

impl<C: Component> Component for Row<C> {
    fn render(&self, context: &Context) {
        let mut offset_context = context.offset(Dimensions::default());
        for child in self.children.iter() {
            child.render(&offset_context);
            offset_context.offset.horizontal += child.measure(context).horizontal;
        }
    }

    fn measure(&self, context: &Context) -> Dimensions {
        let child_dimensions = self.children.iter().map(|c| c.measure(context));
        // Width of a row is the total width of all children.
        let horizontal = child_dimensions.clone().map(|d| d.horizontal).sum();
        // Height of a row is the height of the tallest child.
        let vertical = child_dimensions
            .map(|d| d.vertical)
            .max()
            .unwrap_or_default();
        Dimensions {
            horizontal,
            vertical,
        }
    }
}
