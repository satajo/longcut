use crate::component::Component;
use crate::context::Context;
use crate::model::color::Color;
use crate::model::dimensions::Dimensions;
use crate::model::font::Font;
use crate::model::position::Position;
use crate::model::unit::Unit;

//-----------------------------------------------------------------------------
// Declarations
//-----------------------------------------------------------------------------

pub trait Property<C: Component> {
    fn background(self, color: Color) -> Background<C>;

    fn border(self, thickness: Unit, color: Color) -> Border<C>;

    fn font_style(self, font: Font) -> FontStyle<C>;

    fn foreground(self, color: Color) -> Foreground<C>;

    fn height(self, amount: Unit) -> Height<C>;

    fn height_max(self, amount: Unit) -> MaximumHeight<C>;

    fn height_min(self, amount: Unit) -> MinimumHeight<C>;

    fn margin(self, amount: Unit) -> Margin<C>;

    fn margin_horizontal(self, amount: Unit) -> MarginHorizontal<C>;

    fn margin_vertical(self, amount: Unit) -> MarginVertical<C>;

    fn margin_top(self, amount: Unit) -> MarginTop<C>;

    fn margin_bottom(self, amount: Unit) -> MarginBottom<C>;

    fn margin_left(self, amount: Unit) -> MarginLeft<C>;

    fn margin_right(self, amount: Unit) -> MarginRight<C>;

    fn width(self, amount: Unit) -> Width<C>;

    fn width_max(self, amount: Unit) -> MaximumWidth<C>;

    fn width_min(self, amount: Unit) -> MinimumWidth<C>;
}

impl<C: Component> Property<C> for C {
    fn background(self, color: Color) -> Background<C> {
        Background { child: self, color }
    }

    fn border(self, thickness: Unit, color: Color) -> Border<C> {
        self.margin(thickness).background(color)
    }

    fn font_style(self, font: Font) -> FontStyle<C> {
        FontStyle { child: self, font }
    }

    fn foreground(self, color: Color) -> Foreground<C> {
        Foreground { child: self, color }
    }

    fn height(self, amount: Unit) -> Height<C> {
        self.height_min(amount).height_max(amount)
    }

    fn height_max(self, amount: Unit) -> MaximumHeight<C> {
        MaximumHeight(self, amount)
    }

    fn height_min(self, amount: Unit) -> MinimumHeight<C> {
        MinimumHeight(self, amount)
    }

    fn margin(self, amount: Unit) -> Margin<C> {
        self.margin_horizontal(amount).margin_vertical(amount)
    }

    fn margin_horizontal(self, amount: Unit) -> MarginHorizontal<C> {
        self.margin_left(amount).margin_right(amount)
    }

    fn margin_vertical(self, amount: Unit) -> MarginVertical<C> {
        self.margin_bottom(amount).margin_top(amount)
    }

    fn margin_top(self, amount: Unit) -> MarginTop<C> {
        MarginTop(self, amount)
    }

    fn margin_bottom(self, amount: Unit) -> MarginBottom<C> {
        MarginBottom(self, amount)
    }

    fn margin_left(self, amount: Unit) -> MarginLeft<C> {
        MarginLeft(self, amount)
    }

    fn margin_right(self, amount: Unit) -> MarginRight<C> {
        MarginRight(self, amount)
    }

    fn width(self, amount: Unit) -> Width<C> {
        self.width_min(amount).width_max(amount)
    }

    fn width_max(self, amount: Unit) -> MaximumWidth<C> {
        MaximumWidth(self, amount)
    }

    fn width_min(self, amount: Unit) -> MinimumWidth<C> {
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

pub struct FontStyle<C: Component> {
    font: Font,
    child: C,
}

impl<C: Component> Component for FontStyle<C> {
    fn render(&self, ctx: &Context) {
        ctx.with_font(&self.font, |ctx| {
            self.child.render(ctx);
        })
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        ctx.with_font(&self.font, |ctx| self.child.measure(ctx))
    }
}

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

//-----------------------------------------------------------------------------
// Margin
//-----------------------------------------------------------------------------

pub type Margin<C> = MarginVertical<MarginHorizontal<C>>;

pub type MarginVertical<C> = MarginTop<MarginBottom<C>>;

pub type MarginHorizontal<C> = MarginRight<MarginLeft<C>>;

pub struct MarginTop<C: Component>(C, Unit);

impl<C: Component> Component for MarginTop<C> {
    fn render(&self, ctx: &Context) {
        let amount_px = self.1.as_pixel(ctx);
        let offset = Position::new(0, amount_px);
        let region = Dimensions::new(
            ctx.region.width,
            ctx.region.height.saturating_sub(amount_px),
        );
        ctx.with_subregion(offset, region, |ctx| self.0.render(ctx))
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let amount_px = self.1.as_pixel(ctx);
        Dimensions::new(0, amount_px) + self.0.measure(ctx)
    }
}

pub struct MarginBottom<C: Component>(C, Unit);

impl<C: Component> Component for MarginBottom<C> {
    fn render(&self, ctx: &Context) {
        let amount_px = self.1.as_pixel(ctx);
        let offset = Position::zero();
        let region = Dimensions::new(
            ctx.region.width,
            ctx.region.height.saturating_sub(amount_px),
        );
        ctx.with_subregion(offset, region, |ctx| self.0.render(ctx));
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let amount_px = self.1.as_pixel(ctx);
        Dimensions::new(0, amount_px) + self.0.measure(ctx)
    }
}

pub struct MarginLeft<C: Component>(C, Unit);

impl<C: Component> Component for MarginLeft<C> {
    fn render(&self, ctx: &Context) {
        let amount_px = self.1.as_pixel(ctx);
        let offset = Position::new(amount_px, 0);
        let region = Dimensions::new(
            ctx.region.width.saturating_sub(amount_px),
            ctx.region.height,
        );
        ctx.with_subregion(offset, region, |ctx| self.0.render(ctx))
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let amount_px = self.1.as_pixel(ctx);
        Dimensions::new(amount_px, 0) + self.0.measure(ctx)
    }
}

pub struct MarginRight<C: Component>(C, Unit);

impl<C: Component> Component for MarginRight<C> {
    fn render(&self, ctx: &Context) {
        let amount_px = self.1.as_pixel(ctx);
        let offset = Position::zero();
        let region = Dimensions::new(
            ctx.region.width.saturating_sub(amount_px),
            ctx.region.height,
        );
        ctx.with_subregion(offset, region, |ctx| self.0.render(ctx))
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let amount_px = self.1.as_pixel(ctx);
        Dimensions::new(amount_px, 0) + self.0.measure(ctx)
    }
}

//-----------------------------------------------------------------------------
// Height
//-----------------------------------------------------------------------------

pub type Height<C> = MaximumHeight<MinimumHeight<C>>;

pub struct MaximumHeight<C: Component>(C, Unit);

impl<C: Component> Component for MaximumHeight<C> {
    fn render(&self, ctx: &Context) {
        let amount_px = self.1.as_pixel(ctx);
        let offset = Position::zero();
        let region = Dimensions::new(ctx.region.width, amount_px);
        ctx.with_subregion(offset, region, |ctx| self.0.render(ctx));
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let amount_px = self.1.as_pixel(ctx);
        let child_dimensions = self.0.measure(ctx);
        Dimensions::new(
            child_dimensions.width,
            child_dimensions.height.min(amount_px),
        )
    }
}

pub struct MinimumHeight<C: Component>(C, Unit);

impl<C: Component> Component for MinimumHeight<C> {
    fn render(&self, ctx: &Context) {
        self.0.render(ctx);
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let amount_px = self.1.as_pixel(ctx);
        let child_dimensions = self.0.measure(ctx);
        Dimensions::new(
            child_dimensions.width,
            child_dimensions.height.max(amount_px),
        )
    }
}

//-----------------------------------------------------------------------------
// Width
//-----------------------------------------------------------------------------

pub type Width<C> = MaximumWidth<MinimumWidth<C>>;

pub struct MaximumWidth<C: Component>(C, Unit);

impl<C: Component> Component for MaximumWidth<C> {
    fn render(&self, ctx: &Context) {
        let amount_px = self.1.as_pixel(ctx);
        let offset = Position::zero();
        let region = Dimensions::new(amount_px, ctx.region.height);
        ctx.with_subregion(offset, region, |ctx| self.0.render(ctx));
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let amount_px = self.1.as_pixel(ctx);
        let child_dimensions = self.0.measure(ctx);
        Dimensions::new(
            child_dimensions.width.min(amount_px),
            child_dimensions.height,
        )
    }
}

pub struct MinimumWidth<C: Component>(C, Unit);

impl<C: Component> Component for MinimumWidth<C> {
    fn render(&self, ctx: &Context) {
        self.0.render(ctx);
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        let amount_px = self.1.as_pixel(ctx);
        let child_dimensions = self.0.measure(ctx);

        Dimensions::new(
            child_dimensions.width.max(amount_px),
            child_dimensions.height,
        )
    }
}
