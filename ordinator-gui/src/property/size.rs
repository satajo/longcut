use crate::model::dimensions::Dimensions;
use crate::{Component, Context};

pub struct Height<C: Component>(C, u32);

impl<C: Component> Component for Height<C> {
    fn render(&self, ctx: &Context) {
        self.0.render(ctx);
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let child_dimensions = self.0.measure(ctx);
        Dimensions::new(child_dimensions.width, self.1)
    }
}

pub struct MaximumHeight<C: Component>(C, u32);

impl<C: Component> Component for MaximumHeight<C> {
    fn render(&self, ctx: &Context) {
        self.0.render(ctx);
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let child_dimensions = self.0.measure(ctx);
        Dimensions::new(child_dimensions.width, child_dimensions.height.min(self.1))
    }
}

pub struct MinimumHeight<C: Component>(C, u32);

impl<C: Component> Component for MinimumHeight<C> {
    fn render(&self, ctx: &Context) {
        self.0.render(ctx);
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let child_dimensions = self.0.measure(ctx);
        Dimensions::new(child_dimensions.width, child_dimensions.height.max(self.1))
    }
}

pub struct Width<C: Component>(C, u32);

impl<C: Component> Component for Width<C> {
    fn render(&self, ctx: &Context) {
        self.0.render(ctx);
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let child_dimensions = self.0.measure(ctx);
        Dimensions::new(self.1, child_dimensions.height)
    }
}

pub struct MaximumWidth<C: Component>(C, u32);

impl<C: Component> Component for MaximumWidth<C> {
    fn render(&self, ctx: &Context) {
        self.0.render(ctx);
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let child_dimensions = self.0.measure(ctx);
        Dimensions::new(child_dimensions.width.min(self.1), child_dimensions.height)
    }
}

pub struct MinimumWidth<C: Component>(C, u32);

impl<C: Component> Component for MinimumWidth<C> {
    fn render(&self, ctx: &Context) {
        self.0.render(ctx);
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let child_dimensions = self.0.measure(ctx);
        Dimensions::new(child_dimensions.width.max(self.1), child_dimensions.height)
    }
}

pub trait Size<C: Component> {
    fn height(self, amount: u32) -> Height<C>;

    fn max_height(self, amount: u32) -> MaximumHeight<C>;

    fn min_height(self, amount: u32) -> MinimumHeight<C>;

    fn width(self, amount: u32) -> Width<C>;

    fn max_width(self, amount: u32) -> MaximumWidth<C>;

    fn min_width(self, amount: u32) -> MinimumWidth<C>;
}

impl<C: Component> Size<C> for C {
    fn height(self, amount: u32) -> Height<C> {
        Height(self, amount)
    }

    fn max_height(self, amount: u32) -> MaximumHeight<C> {
        MaximumHeight(self, amount)
    }

    fn min_height(self, amount: u32) -> MinimumHeight<C> {
        MinimumHeight(self, amount)
    }

    fn width(self, amount: u32) -> Width<C> {
        Width(self, amount)
    }

    fn max_width(self, amount: u32) -> MaximumWidth<C> {
        MaximumWidth(self, amount)
    }

    fn min_width(self, amount: u32) -> MinimumWidth<C> {
        MinimumWidth(self, amount)
    }
}
