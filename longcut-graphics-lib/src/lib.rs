use crate::component::Component;
use crate::context::Context;
use crate::model::color::Color;
use crate::model::dimensions::Dimensions;
use crate::model::font::Font;
use crate::model::position::Position;
use crate::port::renderer::Renderer;

pub mod component;
pub mod context;
pub mod model;
pub mod port;
pub mod property;

pub fn render_component(renderer: &dyn Renderer, region: Dimensions, component: impl Component) {
    let initial_color = Color::rgb(0, 0, 0);
    let initial_font = Font::new("sans-serif".to_string(), 16);
    let ctx = Context::new(renderer, &initial_color, &initial_font, region);
    component.render(&ctx);
}
