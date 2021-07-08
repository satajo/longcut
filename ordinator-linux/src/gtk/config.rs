pub struct Config {
    pub padding: u16,
}

impl Config {
    pub fn new() -> Self {
        Self { padding: 16 }
    }
}
