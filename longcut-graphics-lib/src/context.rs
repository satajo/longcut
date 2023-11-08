use crate::model::font::Font;
use crate::port::renderer::Renderer;
use crate::{Color, Dimensions, Position};

pub struct Context<'a> {
    pub color: &'a Color,
    pub font: &'a Font,
    pub offset: Position,
    pub region: Dimensions,
    renderer: &'a dyn Renderer,
}

impl<'a> Context<'a> {
    pub fn new(
        renderer: &'a dyn Renderer,
        color: &'a Color,
        font: &'a Font,
        region: Dimensions,
    ) -> Self {
        Self {
            color,
            font,
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
        self.renderer
            .draw_text(self.color, &self.offset, self.font, text)
    }

    pub fn measure_text(&self, text: &str) -> Dimensions {
        self.renderer.measure_text(self.font, text)
    }

    pub fn with_color(&self, color: &'a Color, f: impl FnOnce(&Self)) {
        f(&Self {
            color,
            font: self.font,
            offset: self.offset,
            region: self.region,
            renderer: self.renderer,
        });
    }

    pub fn with_subregion(&self, offset: Position, region: Dimensions, f: impl FnOnce(&Self)) {
        let final_offset = self.offset + offset;
        // The subregion, after factoring in the offset, must still fit within the current region.
        let final_region = region.intersect(&Dimensions::new(
            self.region.width.saturating_sub(offset.horizontal),
            self.region.height.saturating_sub(offset.vertical),
        ));

        if final_region.width == 0 || final_region.height == 0 {
            // We can never draw anything when one of the dimensions is zero. Aborting.
            return;
        }

        f(&Self {
            color: self.color,
            font: self.font,
            offset: final_offset,
            region: final_region,
            renderer: self.renderer,
        })
    }

    pub fn with_font<V>(&self, font: &'a Font, f: impl FnOnce(&Self) -> V) -> V {
        f(&Self {
            color: self.color,
            font,
            offset: self.offset,
            region: self.region,
            renderer: self.renderer,
        })
    }
}
