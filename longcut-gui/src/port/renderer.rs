use crate::model::alignment::Alignment2d;
use crate::model::color::Color;
use crate::model::dimensions::Dimensions;
use crate::model::position::Position;

pub trait Graphics {
    fn show_gui(&self, requested_properties: WindowProperties, callback: RenderPassFn);

    fn hide_gui(&self);
}

#[derive(Debug, Clone)]
pub struct WindowProperties {
    pub size: Dimensions,
    pub alignment: Alignment2d,
}

pub type RenderPassFn = Box<dyn FnOnce(Dimensions, &dyn Renderer) + Send>;

pub trait Renderer {
    fn draw_rectangle(&self, color: &Color, position: &Position, size: &Dimensions);

    fn draw_text(&self, color: &Color, position: &Position, text: &str);

    fn measure_text(&self, text: &str) -> Dimensions;
}
