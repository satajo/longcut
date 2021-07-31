use std::convert::TryFrom;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KeyPress {
    Character(char),
    Symbol(Symbol),
}

impl KeyPress {
    pub fn from_symbol(symbol: Symbol) -> Self {
        Self::Symbol(symbol)
    }

    pub fn from_character(character: char) -> Self {
        Self::Character(character)
    }
}

impl TryFrom<&str> for KeyPress {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.chars().count() == 1 {
            Ok(Self::from_character(value.chars().next().unwrap()))
        } else if let Ok(symbol) = Symbol::try_from(value) {
            Ok(Self::from_symbol(symbol))
        } else {
            Err("Value does not match any known key")
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Symbol {
    Alt,
    BackSpace,
    Break,
    CapsLock,
    Control,
    Down,
    End,
    Enter,
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
    Shift,
    Space,
    Super,
    Tab,
    Up,
}

impl TryFrom<&str> for Symbol {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Alt" => Ok(Self::Alt),
            "BackSpace" => Ok(Self::BackSpace),
            "Break" => Ok(Self::Break),
            "CapsLock" => Ok(Self::CapsLock),
            "Control" => Ok(Self::Control),
            "Down" => Ok(Self::Down),
            "End" => Ok(Self::End),
            "Enter" => Ok(Self::Enter),
            "Escape" => Ok(Self::Escape),
            "F1" => Ok(Self::F1),
            "F2" => Ok(Self::F2),
            "F3" => Ok(Self::F3),
            "F4" => Ok(Self::F4),
            "F5" => Ok(Self::F5),
            "F6" => Ok(Self::F6),
            "F7" => Ok(Self::F7),
            "F8" => Ok(Self::F8),
            "F9" => Ok(Self::F9),
            "F10" => Ok(Self::F10),
            "F11" => Ok(Self::F11),
            "F12" => Ok(Self::F12),
            "Fn" => Ok(Self::Fn),
            "Home" => Ok(Self::Home),
            "Insert" => Ok(Self::Insert),
            "Left" => Ok(Self::Left),
            "Menu" => Ok(Self::Menu),
            "NumLock" => Ok(Self::NumLock),
            "PageDown" => Ok(Self::PageDown),
            "PageUp" => Ok(Self::PageUp),
            "Pause" => Ok(Self::Pause),
            "PrintScreen" => Ok(Self::PrintScreen),
            "Right" => Ok(Self::Right),
            "ScrollLock" => Ok(Self::ScrollLock),
            "Shift" => Ok(Self::Shift),
            "Space" => Ok(Self::Space),
            "Super" => Ok(Self::Super),
            "Tab" => Ok(Self::Tab),
            "Up" => Ok(Self::Up),
            _ => Err("Value does not match any known symbol"),
        }
    }
}
