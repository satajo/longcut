use std::collections::BTreeSet;
use std::convert::TryFrom;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Key {
    pub symbol: Symbol,
    pub modifiers: BTreeSet<Modifier>,
}

impl Key {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            modifiers: BTreeSet::new(),
        }
    }

    pub fn add_modifier(&mut self, modifier: Modifier) {
        self.modifiers.insert(modifier);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Modifier {
    Alt,
    Control,
    Shift,
    Super,
}

impl TryFrom<&str> for Modifier {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "alt" => Ok(Self::Alt),
            "control" => Ok(Self::Control),
            "shift" => Ok(Self::Shift),
            "super" => Ok(Self::Super),
            _ => Err("value is not a valid symbol"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Symbol {
    Character(char),
    // Named special characters
    AltL,
    AltR,
    BackSpace,
    Break,
    CapsLock,
    Control,
    Down,
    End,
    Return,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Fn,
    Home,
    Insert,
    Left,
    Menu,
    NumLock,
    PageDown,
    PageUp,
    Pause,
    PrintScreen,
    Right,
    ScrollLock,
    ShiftL,
    ShiftR,
    Space,
    SuperL,
    SuperR,
    Tab,
    Up,
}

impl TryFrom<&str> for Symbol {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "alt_l" => Ok(Self::AltL),
            "alt_r" => Ok(Self::AltR),
            "backspace" => Ok(Self::BackSpace),
            "break" => Ok(Self::Break),
            "capslock" => Ok(Self::CapsLock),
            "control_l" => Ok(Self::Control),
            "control_r" => Ok(Self::Control),
            "down" => Ok(Self::Down),
            "end" => Ok(Self::End),
            "escape" => Ok(Self::Escape),
            "f1" => Ok(Self::F1),
            "f2" => Ok(Self::F2),
            "f3" => Ok(Self::F3),
            "f4" => Ok(Self::F4),
            "f5" => Ok(Self::F5),
            "f6" => Ok(Self::F6),
            "F7" => Ok(Self::F7),
            "f8" => Ok(Self::F8),
            "f9" => Ok(Self::F9),
            "f10" => Ok(Self::F10),
            "f11" => Ok(Self::F11),
            "f12" => Ok(Self::F12),
            "fn" => Ok(Self::Fn),
            "home" => Ok(Self::Home),
            "insert" => Ok(Self::Insert),
            "left" => Ok(Self::Left),
            "menu" => Ok(Self::Menu),
            "numlock" => Ok(Self::NumLock),
            "pagedown" => Ok(Self::PageDown),
            "pageup" => Ok(Self::PageUp),
            "pause" => Ok(Self::Pause),
            "printscreen" => Ok(Self::PrintScreen),
            "right" => Ok(Self::Right),
            "enter" => Ok(Self::Return), // Common alias
            "return" => Ok(Self::Return),
            "scrolllock" => Ok(Self::ScrollLock),
            "shift_l" => Ok(Self::ShiftL),
            "shift_r" => Ok(Self::ShiftR),
            "space" => Ok(Self::Space),
            "super_l" => Ok(Self::SuperL),
            "super_r" => Ok(Self::SuperR),
            "tab" => Ok(Self::Tab),
            "up" => Ok(Self::Up),
            otherwise => {
                if otherwise.chars().count() == 1 {
                    Ok(Self::Character(otherwise.chars().next().unwrap()))
                } else {
                    println!("Could not match: {:?}", otherwise);
                    Err("value is not a valid symbol")
                }
            }
        }
    }
}
