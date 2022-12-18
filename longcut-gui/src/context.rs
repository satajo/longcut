use crate::port::renderer::Renderer;
use crate::{Color, Dimensions, Position};

pub struct Context<'a> {
    pub color: &'a Color,
    pub offset: Position,
    pub region: Dimensions,
    pub renderer: &'a dyn Renderer,
}

impl<'a> Context<'a> {
    pub fn new(renderer: &'a dyn Renderer, color: &'a Color, region: Dimensions) -> Self {
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
