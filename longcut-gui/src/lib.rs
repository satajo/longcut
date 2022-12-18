use crate::component::Component;
use crate::config::Config;
use crate::context::Context;
use crate::model::color::Color;
use crate::model::dimensions::Dimensions;
use crate::model::position::Position;
use crate::port::renderer::Graphics;

pub mod adapter;
mod component;
pub mod config;
mod context;
pub mod model;
pub mod port;
mod property;
pub mod screen;

pub struct GuiModule<'a> {
    graphics: &'a dyn Graphics,
    config: Config,
}

impl<'a> GuiModule<'a> {
    pub fn new(graphics: &'a impl Graphics, config: Config) -> Self {
        Self { graphics, config }
    }

    pub fn display_gui(&self, assembly_fn: Box<dyn FnOnce() -> Box<dyn Component> + Send>) {
        let requested_properties = self.config.window_properties.clone();
        let initial_color = self.config.theme.background_color.clone();
        self.graphics.show_gui(
            requested_properties,
            Box::new(move |realized_dimensions, renderer| {
                let ctx = Context::new(renderer, &initial_color, realized_dimensions);
                let component = assembly_fn();
                component.render(&ctx);
            }),
        );
    }

    pub fn hide_gui(&self) {
        self.graphics.hide_gui();
    }
}
