use longcut_core::model::key::{Key, Modifier};
use longcut_graphics_lib::component::Component;
use longcut_graphics_lib::component::text::Text;
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq)]
pub struct Shortcut {
    modifiers: String,
    symbol: String,
}

impl Ord for Shortcut {
    fn cmp(&self, other: &Self) -> Ordering {
        self.symbol
            .cmp(&other.symbol)
            .then_with(|| self.modifiers.cmp(&other.modifiers))
    }
}

impl PartialOrd for Shortcut {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Shortcut {
    pub fn new(key: &Key) -> Self {
        let mut modifiers = String::new();

        if key.modifiers.contains(&Modifier::Shift) {
            modifiers += "s-";
        }

        if key.modifiers.contains(&Modifier::Control) {
            modifiers += "c-";
        }

        if key.modifiers.contains(&Modifier::Alt) {
            modifiers += "a-";
        }

        if key.modifiers.contains(&Modifier::Super) {
            modifiers += "u-";
        }

        Self {
            modifiers,
            symbol: key.symbol.to_string(),
        }
    }

    pub fn assemble(&self) -> impl Component + use<> {
        let mut text = self.modifiers.clone();
        text.push_str(&self.symbol);
        Text::new(text)
    }
}
