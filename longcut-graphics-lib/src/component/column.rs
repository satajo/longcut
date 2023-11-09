use crate::component::Component;
use crate::context::Context;
use crate::model::dimensions::Dimensions;
use crate::model::position::Position;
use crate::model::unit::Unit;
use crate::property::{MarginBottom, Property};

#[derive(Default)]
pub struct Column<C: Component> {
    children: Vec<C>,
}

impl<C: Component> Column<C> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn gap_size(self, amount: Unit) -> Column<MarginBottom<C>> {
        let padded_children = self
            .children
            .into_iter()
            .map(|child| child.margin_bottom(amount))
            .collect();

        Column {
            children: padded_children,
        }
    }

    pub fn add_child(mut self, child: C) -> Self {
        self.children.push(child);
        self
    }
}

impl<C: Component> Component for Column<C> {
    fn render(&self, ctx: &Context) {
        let mut offset = Position::zero();
        for child in self.children.iter() {
            let child_height = child.measure(ctx).height;
            let region = Dimensions::new(ctx.region.width, child_height);
            ctx.with_subregion(offset, region, |ctx| child.render(ctx));
            offset.vertical += child_height;
        }
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let child_dimensions = self.children.iter().map(|c| c.measure(ctx));

        // Width of a column is the width of the widest child.
        let width = child_dimensions.clone().map(|d| d.width).max().unwrap_or(0);

        // Height of a column is the total height of all children.
        let height = child_dimensions.map(|d| d.height).sum();

        Dimensions::new(width, height)
    }
}
