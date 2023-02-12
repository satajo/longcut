use longcut_graphics_lib::model::color::Color;
use longcut_graphics_lib::model::font::Font;

#[derive(Debug, Clone)]
pub struct Theme {
    pub font: Font,

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
