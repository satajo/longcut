use crate::model::theme::Theme;
use crate::model::window_properties::WindowProperties;
use longcut_graphics_lib::model::alignment::{Alignment, Alignment2d};
use longcut_graphics_lib::model::color::Color;
use longcut_graphics_lib::model::dimensions::Dimensions;
use longcut_graphics_lib::model::font::Font;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(try_from = "ConfigSchema")]
pub struct Config {
    pub theme: Theme,
    pub window_properties: WindowProperties,
}

/// One flat struct for the whole section: serde cannot reject unknown fields across a
/// `flatten` boundary, so nesting the theme and window schemas would silently accept typos.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigSchema {
    font_family: String,
    font_size: u8,

    background_color: Option<ColorSchema>,
    border_color: Option<ColorSchema>,
    foreground_color: Option<ColorSchema>,
    placeholder_color: Option<ColorSchema>,
    error_background_color: Option<ColorSchema>,
    error_border_color: Option<ColorSchema>,
    error_foreground_color: Option<ColorSchema>,
    action_branch_color: Option<ColorSchema>,
    action_execute_color: Option<ColorSchema>,
    action_system_color: Option<ColorSchema>,

    window_width: u32,
    window_height: u32,
}

impl TryFrom<ConfigSchema> for Config {
    type Error = String;

    fn try_from(value: ConfigSchema) -> Result<Self, Self::Error> {
        let font = parse_font(value.font_family, value.font_size)?;

        let default_background_color = Color::rgb(26, 26, 26);
        let default_border_color = Color::rgb(68, 68, 68);
        let default_foreground_color = Color::rgb(229, 229, 229);

        let background_color = parse_color(value.background_color, &default_background_color)?;
        let border_color = parse_color(value.border_color, &default_border_color)?;
        let foreground_color = parse_color(value.foreground_color, &default_foreground_color)?;

        let error_background_color = parse_color(value.error_background_color, &background_color)?;
        let error_border_color = parse_color(value.error_border_color, &border_color)?;
        let error_foreground_color = parse_color(value.error_foreground_color, &foreground_color)?;

        let action_branch_color = parse_color(value.action_branch_color, &foreground_color)?;
        let action_execute_color = parse_color(value.action_execute_color, &foreground_color)?;
        let action_system_color = parse_color(value.action_system_color, &foreground_color)?;
        let placeholder_color = parse_color(value.placeholder_color, &foreground_color)?;

        let theme = Theme {
            font,
            background_color,
            border_color,
            foreground_color,
            error_background_color,
            error_border_color,
            error_foreground_color,
            action_branch_color,
            action_execute_color,
            action_system_color,
            placeholder_color,
        };

        let window_properties = WindowProperties {
            size: Dimensions::new(value.window_width, value.window_height),
            alignment: Alignment2d::new(Alignment::Center, Alignment::End),
        };

        Ok(Config {
            theme,
            window_properties,
        })
    }
}

fn parse_font(family: String, size: u8) -> Result<Font, String> {
    if size == 0 {
        return Err("The font size must be larger than 0".to_string());
    }
    if family.is_empty() {
        return Err("The font_family can not be an empty string".to_string());
    }
    Ok(Font::new(family, size))
}

/// Falls back to `default` when the color is not configured.
fn parse_color(value: Option<ColorSchema>, default: &Color) -> Result<Color, String> {
    match value {
        None => Ok(default.clone()),
        Some(color) => color.try_into(),
    }
}

#[derive(Debug, Deserialize)]
struct ColorSchema(String);

impl TryFrom<ColorSchema> for Color {
    type Error = String;

    fn try_from(value: ColorSchema) -> Result<Self, Self::Error> {
        let hex_string = value.0;
        Color::try_from_hex(&hex_string)
            .map_err(|_| format!("Could not parse {hex_string} as a valid color!"))
    }
}
