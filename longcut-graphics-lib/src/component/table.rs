use crate::component::column::Column;
use crate::component::row::Row;
use crate::component::Component;
use crate::context::Context;
use crate::model::dimensions::Dimensions;
use crate::model::unit::Unit;
use crate::property::Property;

pub struct Table<C: Component> {
    column_width: u32,
    children: Vec<C>,
}

impl<C: Component> Table<C> {
    pub fn new(column_width: u32) -> Self {
        Self {
            column_width,
            children: Vec::new(),
        }
    }

    pub fn add_child(mut self, child: C) -> Self {
        self.children.push(child);
        self
    }

    fn column_count(&self, ctx: &Context) -> usize {
        (ctx.region.width / self.column_width).max(1) as usize
    }
}

impl<C: Component> Component for Table<C> {
    fn render(&self, ctx: &Context) {
        let mut rows = Column::new();
        let column_count = self.column_count(ctx);
        let cell_width = Unit::Px((ctx.region.width as f32 / column_count as f32) as u32);

        for row_items in self.children.chunks(self.column_count(ctx)) {
            let mut row = Row::new();

            for item in row_items {
                row = row.add_child(item.width(cell_width));
            }

            rows = rows.add_child(row);
        }

        rows.render(ctx);
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let total_height: u32 = self
            .children
            .chunks(self.column_count(ctx))
            .map(|row| -> u32 {
                row.iter()
                    .map(|cell| cell.measure(ctx).height)
                    .max()
                    .unwrap()
            })
            .sum();

        Dimensions::new(ctx.region.width, total_height)
    }
}
