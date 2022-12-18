use crate::model::alignment::{Alignment, Alignment2d};
use crate::model::color::Color;
use crate::model::dimensions::Dimensions;
use crate::model::theme::Theme;
use crate::port::renderer::WindowProperties;

#[derive(Debug, Clone)]
pub struct Config {
    pub theme: Theme,
    pub window_properties: WindowProperties,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme {
                background_color: Color::rgb(38, 38, 38),
                border_color: Color::rgb(77, 77, 77),
                foreground_color: Color::rgb(229, 229, 229),

                error_background_color: Color::rgb(67, 2, 11),
                error_border_color: Color::rgb(127, 6, 38),
                error_foreground_color: Color::rgb(229, 229, 229),

                action_branch_color: Color::rgb(238, 118, 0),
                action_execute_color: Color::rgb(229, 229, 229),
                action_system_color: Color::rgb(127, 127, 127),

                placeholder_color: Color::rgb(127, 127, 127),
            },
            window_properties: WindowProperties {
                size: Dimensions::new(1280, 360),
                alignment: Alignment2d::new(Alignment::Center, Alignment::End),
            },
        }
    }
}
