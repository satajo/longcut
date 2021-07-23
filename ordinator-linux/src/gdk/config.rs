use gdk::cairo::Context;
use std::ops::Add;

pub struct Color {
    red: f64,
    green: f64,
    blue: f64,
    // 0 = transparent, 1 = opaque
    alpha: f64,
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

    pub fn apply(&self, cr: &Context) {
        cr.set_source_rgba(self.red, self.green, self.blue, self.alpha);
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Dimensions {
    pub horizontal: u32,
    pub vertical: u32,
}

impl Dimensions {
    pub fn new(horizontal: u32, vertical: u32) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}

impl Add for Dimensions {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            horizontal: self.horizontal + rhs.horizontal,
            vertical: self.vertical + rhs.vertical,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub enum Alignment {
    Beginning,
    Center,
    End,
}

pub struct WindowConfig {
    pub size: Dimensions,
    pub horizontal: Alignment,
    pub vertical: Alignment,
}

pub struct Config {
    pub color_bg: Color,
    pub color_fg: Color,
    pub window: WindowConfig,
}
