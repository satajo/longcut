use crate::component::Component;
use crate::component::column::Column;
use crate::component::row::Row;
use crate::context::Context;
use crate::model::dimensions::Dimensions;
use crate::model::unit::Unit;
use crate::property::Property;
use std::num::NonZeroU32;

#[derive(Debug)]
pub struct Table<C: Component> {
    column_width: NonZeroU32,
    children: Vec<C>,
}

impl<C: Component> Table<C> {
    #[must_use]
    pub fn new(column_width: NonZeroU32) -> Self {
        Self {
            column_width,
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn add_child(mut self, child: C) -> Self {
        self.children.push(child);
        self
    }

    /// A region narrower than one column still holds one column.
    fn column_count(&self, ctx: &Context) -> NonZeroU32 {
        NonZeroU32::new(ctx.region.width / self.column_width).unwrap_or(NonZeroU32::MIN)
    }

    /// The children in the order they are laid out, one slice per row.
    fn rows(&self, ctx: &Context) -> impl Iterator<Item = &[C]> {
        let column_count = usize::try_from(self.column_count(ctx).get())
            .expect("every target longcut builds for has a usize of at least 32 bits");
        self.children.chunks(column_count)
    }
}

impl<C: Component> Component for Table<C> {
    fn render(&self, ctx: &Context) {
        let mut rows = Column::new();
        let cell_width = Unit::Px(ctx.region.width / self.column_count(ctx));

        for row_items in self.rows(ctx) {
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
            .rows(ctx)
            .map(|row| -> u32 {
                row.iter()
                    .map(|cell| cell.measure(ctx).height)
                    .fold(0, u32::max)
            })
            .fold(0, u32::saturating_add);

        Dimensions::new(ctx.region.width, total_height)
    }
}
