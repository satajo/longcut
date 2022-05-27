use ordinator_core::model::key::{Key, Modifier, Symbol};
use ordinator_gui::component::text::Text;
use ordinator_gui::Component;

#[derive(Debug)]
pub struct Shortcut {
    text: String,
}

impl Shortcut {
    pub fn new(key: &Key) -> Self {
        let mut text = String::new();

        if key.modifiers.contains(&Modifier::Shift) {
            text += "s-";
        }

        if key.modifiers.contains(&Modifier::Control) {
            text += "c-";
        }

        if key.modifiers.contains(&Modifier::Alt) {
            text += "a-";
        }

        if key.modifiers.contains(&Modifier::Super) {
            text += "u-";
        }

        let symbol = match &key.symbol {
            Symbol::Character(c) => c.to_string(),
            otherwise => format!("{:?}", otherwise).to_lowercase(),
        };

        text.push_str(&symbol);

        Self { text }
    }

    pub fn assemble(&self) -> impl Component {
        Text::new(self.text.clone())
    }
}
