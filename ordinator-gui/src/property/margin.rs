use crate::model::dimensions::Dimensions;
use crate::model::position::Position;
use crate::{Component, Context};

pub struct MarginTop<C: Component>(C, u32);

impl<C: Component> Component for MarginTop<C> {
    fn render(&self, ctx: &Context) {
        let offset = Position::new(0, self.1);
        let region = Dimensions::new(ctx.region.width, ctx.region.height - self.1);
        ctx.with_subregion(offset, region, |ctx| self.0.render(ctx))
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        Dimensions::new(0, self.1) + self.0.measure(ctx)
    }
}

pub struct MarginBottom<C: Component>(C, u32);

impl<C: Component> Component for MarginBottom<C> {
    fn render(&self, ctx: &Context) {
        let offset = Position::new(0, 0);
        let region = Dimensions::new(ctx.region.width, ctx.region.height - self.1);
        ctx.with_subregion(offset, region, |ctx| self.0.render(ctx));
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        Dimensions::new(0, self.1) + self.0.measure(ctx)
    }
}

pub struct MarginLeft<C: Component>(C, u32);

impl<C: Component> Component for MarginLeft<C> {
    fn render(&self, ctx: &Context) {
        let offset = Position::new(self.1, 0);
        let region = Dimensions::new(ctx.region.width - self.1, ctx.region.height);
        ctx.with_subregion(offset, region, |ctx| self.0.render(ctx))
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        Dimensions::new(self.1, 0) + self.0.measure(ctx)
    }
}

pub struct MarginRight<C: Component>(C, u32);

impl<C: Component> Component for MarginRight<C> {
    fn render(&self, ctx: &Context) {
        let offset = Position::new(0, 0);
        let region = Dimensions::new(ctx.region.width - self.1, ctx.region.height);
        ctx.with_subregion(offset, region, |ctx| self.0.render(ctx))
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        Dimensions::new(self.1, 0) + self.0.measure(ctx)
    }
}

pub trait Margin<C: Component> {
    fn margin(self, amount: u32) -> MarginTop<MarginBottom<MarginRight<MarginLeft<C>>>>;

    fn margin_top(self, amount: u32) -> MarginTop<C>;

    fn margin_bottom(self, amount: u32) -> MarginBottom<C>;

    fn margin_left(self, amount: u32) -> MarginLeft<C>;

    fn margin_right(self, amount: u32) -> MarginRight<C>;
}

impl<C: Component> Margin<C> for C {
    fn margin(self, amount: u32) -> MarginTop<MarginBottom<MarginRight<MarginLeft<C>>>> {
        self.margin_left(amount)
            .margin_right(amount)
            .margin_bottom(amount)
            .margin_top(amount)
    }

    fn margin_top(self, amount: u32) -> MarginTop<C> {
        MarginTop(self, amount)
    }

    fn margin_bottom(self, amount: u32) -> MarginBottom<C> {
        MarginBottom(self, amount)
    }

    fn margin_left(self, amount: u32) -> MarginLeft<C> {
        MarginLeft(self, amount)
    }

    fn margin_right(self, amount: u32) -> MarginRight<C> {
        MarginRight(self, amount)
    }
}
