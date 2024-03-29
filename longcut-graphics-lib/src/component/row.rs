use crate::component::Component;
use crate::context::Context;
use crate::model::dimensions::Dimensions;
use crate::model::position::Position;
use crate::model::unit::Unit;
use crate::property::{MarginRight, Property};

#[derive(Default)]
pub struct Row<C: Component> {
    children: Vec<C>,
}

impl<C: Component> Row<C> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn gap_size(self, amount: Unit) -> Row<MarginRight<C>> {
        let padded_children = self
            .children
            .into_iter()
            .map(|child| child.margin_right(amount))
            .collect();

        Row {
            children: padded_children,
        }
    }

    pub fn add_child(mut self, child: C) -> Self {
        self.children.push(child);
        self
    }
}

impl<C: Component> Component for Row<C> {
    fn render(&self, ctx: &Context) {
        let mut offset = Position::zero();
        for child in self.children.iter() {
            let child_width = child.measure(ctx).width;
            let region = Dimensions::new(child_width, ctx.region.height);
            ctx.with_subregion(offset, region, |ctx| child.render(ctx));
            offset.horizontal += child_width;
        }
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let child_dimensions = self.children.iter().map(|c| c.measure(ctx));

        // Width of a row is the total width of all children.
        let width = child_dimensions.clone().map(|d| d.width).sum();

        // Height of a row is the height of the tallest child.
        let height = child_dimensions.map(|d| d.height).max().unwrap_or(0);

        Dimensions::new(width, height)
    }
}
