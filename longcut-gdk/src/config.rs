use crate::window;
use longcut_gui::model::alignment::Alignment;
use longcut_gui::model::color::Color;
use longcut_gui::model::dimensions::Dimensions;

pub struct Config {
    pub window: window::Config,
    pub theme: Theme,
}

pub struct Theme {
    // Normal
    pub background_color: Color,
    pub border_color: Color,
    pub foreground_color: Color,
    // Error
    pub error_background_color: Color,
    pub error_border_color: Color,
    pub error_foreground_color: Color,
    // Actions
    pub action_branch_color: Color,
    pub action_execute_color: Color,
    pub action_system_color: Color,
    // Misc
    pub placeholder_color: Color,
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
            window: window::Config {
                size: Dimensions {
                    height: 360,
                    width: 1280,
                },
                horizontal: Alignment::Center,
                vertical: Alignment::End,
            },
        }
    }
}
