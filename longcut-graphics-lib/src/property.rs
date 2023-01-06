use crate::component::Component;
use crate::context::Context;
use crate::model::color::Color;
use crate::model::dimensions::Dimensions;
use crate::model::position::Position;

//-----------------------------------------------------------------------------
// Declarations
//-----------------------------------------------------------------------------

pub trait Property<C: Component> {
    fn background(self, color: Color) -> Background<C>;

    fn border(self, thickness: u32, color: Color) -> Border<C>;

    fn foreground(self, color: Color) -> Foreground<C>;

    fn height(self, amount: u32) -> Height<C>;

    fn height_max(self, amount: u32) -> MaximumHeight<C>;

    fn height_min(self, amount: u32) -> MinimumHeight<C>;

    fn margin(self, amount: u32) -> Margin<C>;

    fn margin_top(self, amount: u32) -> MarginTop<C>;

    fn margin_bottom(self, amount: u32) -> MarginBottom<C>;

    fn margin_left(self, amount: u32) -> MarginLeft<C>;

    fn margin_right(self, amount: u32) -> MarginRight<C>;

    fn width(self, amount: u32) -> Width<C>;

    fn width_max(self, amount: u32) -> MaximumWidth<C>;

    fn width_min(self, amount: u32) -> MinimumWidth<C>;
}

impl<C: Component> Property<C> for C {
    fn background(self, color: Color) -> Background<C> {
        Background { child: self, color }
    }

    fn border(self, thickness: u32, color: Color) -> Border<C> {
        self.margin(thickness).background(color)
    }

    fn foreground(self, color: Color) -> Foreground<C> {
        Foreground { child: self, color }
    }

    fn height(self, amount: u32) -> Height<C> {
        Height(self, amount)
    }

    fn height_max(self, amount: u32) -> MaximumHeight<C> {
        MaximumHeight(self, amount)
    }

    fn height_min(self, amount: u32) -> MinimumHeight<C> {
        MinimumHeight(self, amount)
    }

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

    fn width(self, amount: u32) -> Width<C> {
        Width(self, amount)
    }

    fn width_max(self, amount: u32) -> MaximumWidth<C> {
        MaximumWidth(self, amount)
    }

    fn width_min(self, amount: u32) -> MinimumWidth<C> {
        MinimumWidth(self, amount)
    }
}

//-----------------------------------------------------------------------------
// Definitions
//-----------------------------------------------------------------------------

pub struct Background<C: Component> {
    color: Color,
    child: C,
}

impl<C: Component> Component for Background<C> {
    fn render(&self, ctx: &Context) {
        ctx.with_color(&self.color, |ctx| ctx.draw_rectangle(&ctx.region));
        self.child.render(ctx);
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        self.child.measure(ctx)
    }
}

pub type Border<C> = Background<Margin<C>>;

pub struct Foreground<C: Component> {
    color: Color,
    child: C,
}

impl<C: Component> Component for Foreground<C> {
    fn render(&self, ctx: &Context) {
        ctx.with_color(&self.color, |ctx| {
            self.child.render(ctx);
        });
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        self.child.measure(ctx)
    }
}

pub type Margin<C> = MarginTop<MarginBottom<MarginRight<MarginLeft<C>>>>;

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
