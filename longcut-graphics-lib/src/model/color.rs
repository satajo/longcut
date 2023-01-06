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
}
