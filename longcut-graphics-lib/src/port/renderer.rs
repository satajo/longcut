use crate::model::color::Color;
use crate::model::dimensions::Dimensions;
use crate::model::font::Font;
use crate::model::position::Position;

pub trait Renderer {
    fn draw_rectangle(&self, color: &Color, position: &Position, size: &Dimensions);

    fn draw_text(&self, color: &Color, position: &Position, font: &Font, text: &str);

    fn measure_text(&self, font: &Font, text: &str) -> Dimensions;
}
