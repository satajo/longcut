#[derive(Clone, Debug)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    // 0 = transparent, 1 = opaque
    pub alpha: f64,
}

impl Color {
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red: red as f64 / 255.0,
            green: green as f64 / 255.0,
            blue: blue as f64 / 255.0,
            alpha: 1.0,
        }
    }

    /// Attempts to parse a hexadecimal color string into Self.
    ///
    /// The hex string should start with the # character and use two characters to specify each of
    /// the RGB color components.
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

        fn extract_component(value: &str) -> Result<u8, String> {
            u8::from_str_radix(value, 16)
                .map_err(|_| format!("The value '{value}' is an invalid hex string"))
        }

        if value.len() != 7 {
            return Err("Value is of invalid length".into());
        }

        if !value.starts_with('#') {
            return Err("Value must start with the # character".into());
        }

        let red = extract_component(&value[1..3])?;
        let green = extract_component(&value[3..5])?;
        let blue = extract_component(&value[5..7])?;

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
    }
}
