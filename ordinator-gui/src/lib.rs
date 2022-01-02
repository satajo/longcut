use crate::model::color::Color;
use crate::model::dimensions::Dimensions;
use crate::model::position::Position;
use crate::port::renderer::Renderer;

pub mod component;
pub mod model;
pub mod port;
pub mod property;

pub struct Context<'a> {
    color: &'a Color,
    offset: Position,
    region: Dimensions,
    renderer: &'a dyn Renderer,
}

impl<'a> Context<'a> {
    pub fn new(renderer: &'a impl Renderer, color: &'a Color, region: Dimensions) -> Self {
        Self {
            color,
            offset: Position::new(0, 0),
            region,
            renderer,
        }
    }

    pub fn draw_rectangle(&self, dimensions: &Dimensions) {
        self.renderer
            .draw_rectangle(self.color, &self.offset, dimensions);
    }

    pub fn draw_text(&self, text: &str) {
        self.renderer.draw_text(self.color, &self.offset, text)
    }

    pub fn measure_text(&self, text: &str) -> Dimensions {
        self.renderer.measure_text(text)
    }

    pub fn with_color(&self, color: &'a Color, f: impl FnOnce(&Self)) {
        f(&Self {
            color,
            offset: self.offset,
            region: self.region,
            renderer: self.renderer,
        });
    }

    pub fn with_subregion(&self, offset: Position, region: Dimensions, f: impl FnOnce(&Self)) {
        f(&Self {
            color: self.color,
            offset: self.offset + offset,
            region,
            renderer: self.renderer,
        })
    }
}

pub trait Component {
    fn render(&self, ctx: &Context);

    fn measure(&self, ctx: &Context) -> Dimensions;
}

impl<'a, T: Component> Component for &'a T {
    fn render(&self, ctx: &Context) {
        (*self).render(ctx)
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        (*self).measure(ctx)
    }
}

impl Component for Box<dyn Component> {
    fn render(&self, ctx: &Context) {
        self.as_ref().render(ctx)
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        self.as_ref().measure(ctx)
    }
}
