#[derive(Clone, Debug)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    // 0 = transparent, 1 = opaque
    pub alpha: f64,
}

impl Color {
    #[must_use]
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red: f64::from(red) / 255.0,
            green: f64::from(green) / 255.0,
            blue: f64::from(blue) / 255.0,
            alpha: 1.0,
        }
    }

    /// Attempts to parse a hexadecimal color string into Self.
    ///
    /// The hex string should start with the # character and use two characters to specify each of
    /// the RGB color components.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid 7-character hex color (e.g. `#rrggbb`).
    ///
    /// Example:
    /// ```
    /// use longcut_graphics_lib::model::color::Color;
    ///
    /// let color = Color::try_from_hex("#ffffff");
    /// assert!(color.is_ok())
    /// ```
    pub fn try_from_hex(value: &str) -> Result<Self, String> {
        // TODO: Writing these sorts of conversion functions is most likely not worth it when support
        // for more color formats is desired. When that time comes, use a proper color parsing library.

        let Some(components) = value.strip_prefix('#') else {
            return Err("Value must start with the # character".into());
        };

        let bytes = hex::decode(components).map_err(|error| {
            format!("The value '{components}' is an invalid hex string: {error}")
        })?;
        let [red, green, blue] = <[u8; 3]>::try_from(bytes).map_err(|bytes| {
            format!(
                "The value '{components}' has {} color components instead of 3",
                bytes.len()
            )
        })?;

        Ok(Self::rgb(red, green, blue))
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn can_parse_color_from_a_valid_hex_color_string() {
        let black = Color::try_from_hex("#000000").unwrap();
        assert!(black.red < 0.01);
        assert!(black.green < 0.01);
        assert!(black.blue < 0.01);

        let white = Color::try_from_hex("#ffffff").unwrap();
        assert!(white.red > 0.99);
        assert!(white.green > 0.99);
        assert!(white.blue > 0.99);
    }

    #[test]
    fn can_not_parse_from_an_invalid_length_hex_string() {
        // This could be valid but is not supported.
        assert!(Color::try_from_hex("#fff").is_err());

        // This too could be valid but is not supported.
        assert!(Color::try_from_hex("#ffffff00").is_err());

        // Obviously wrong.
        assert!(Color::try_from_hex("#").is_err());
        assert!(Color::try_from_hex("#aa").is_err());
        assert!(Color::try_from_hex("#abcde").is_err());
    }

    #[test]
    fn can_not_parse_if_hex_string_does_not_start_with_a_hash() {
        // Prefixed with some other character.
        assert!(Color::try_from_hex("$FFFFFF").is_err());

        // Padded at the end.
        assert!(Color::try_from_hex("0000000").is_err());
        assert!(Color::try_from_hex("abababa").is_err());
    }

    #[test]
    fn can_not_parse_if_hex_string_contains_invalid_characters() {
        // Characters beyond the [0 .. F] range
        assert!(Color::try_from_hex("#aabbGG").is_err());
        assert!(Color::try_from_hex("#ffffxx").is_err());

        // Just nonsense characters.
        assert!(Color::try_from_hex("###ffa#").is_err());
        assert!(Color::try_from_hex("#0000/1").is_err());

        // Characters outside ASCII.
        assert!(Color::try_from_hex("#ääbbcc").is_err());
        assert!(Color::try_from_hex("#aabbc€").is_err());
    }
}
