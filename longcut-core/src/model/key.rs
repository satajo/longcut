use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;
use xkbcommon::xkb;

/// A key as the user names it: a symbol with the modifiers held alongside it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Key {
    pub symbol: Symbol,
    pub modifiers: BTreeSet<Modifier>,
}

impl Key {
    #[must_use]
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

/// Renders the key as it is configured: the modifiers, then the symbol, joined with `+`.
impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for modifier in &self.modifiers {
            write!(f, "{modifier}+")?;
        }
        write!(f, "{}", self.symbol)
    }
}

/// What a key means under the active keyboard layout: the character or named key it types.
///
/// A symbol is named the way xkb names keysyms, which is how `xev`, `wev` and
/// `xkbcli how-to-type` print them: `a`, `exclam`, `Escape`, `Caps_Lock`, `Page_Up`.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Symbol(xkb::Keysym);

impl Symbol {
    pub const ESCAPE: Symbol = Symbol(xkb::Keysym::Escape);
    pub const RETURN: Symbol = Symbol(xkb::Keysym::Return);
    pub const BACKSPACE: Symbol = Symbol(xkb::Keysym::BackSpace);

    /// The symbol that types `c`. Unicode noncharacters have none.
    #[must_use]
    pub fn from_char(c: char) -> Option<Self> {
        Self::from_keysym(xkb::Keysym::from_char(c))
    }

    /// The symbol an xkb keysym stands for, for input adapters that resolve keys with xkb.
    /// `NoSymbol` stands for none.
    #[must_use]
    pub fn from_keysym(keysym: xkb::Keysym) -> Option<Self> {
        (keysym != xkb::Keysym::NoSymbol).then_some(Self(keysym))
    }

    /// The xkb keysym the symbol stands for, for input adapters that bind keys with xkb.
    #[must_use]
    pub fn keysym(self) -> xkb::Keysym {
        self.0
    }

    /// The character the symbol types. Control characters and named keys type none.
    #[must_use]
    pub fn character(self) -> Option<char> {
        char::from_u32(xkb::keysym_to_utf32(self.0)).filter(|c| !c.is_control())
    }
}

/// Parses a symbol's name. A single character stands for the symbol that types it; anything else
/// must be an xkb keysym name, spelled exactly as xkb does.
impl FromStr for Symbol {
    type Err = KeyParseError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        let mut chars = name.chars();
        let symbol = match (chars.next(), chars.next()) {
            (Some(c), None) => Self::from_char(c),
            _ => Self::from_keysym(xkb::keysym_from_name(name, xkb::KEYSYM_NO_FLAGS)),
        };
        symbol.ok_or_else(|| KeyParseError::UnknownSymbol(name.to_string()))
    }
}

/// Renders the symbol as it is named: its character when it types a visible one, otherwise its xkb
/// keysym name. A symbol with several xkb names renders under the canonical one, so `Page_Up`
/// renders as `Prior`.
impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.character() {
            Some(c) if !c.is_whitespace() => write!(f, "{c}"),
            _ => f.write_str(&xkb::keysym_get_name(self.0)),
        }
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol({self})")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Modifier {
    Alt,
    Control,
    Shift,
    Super,
}

impl Modifier {
    /// The modifier's xkb name, which is also its configured spelling.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Modifier::Alt => "Alt",
            Modifier::Control => "Control",
            Modifier::Shift => "Shift",
            Modifier::Super => "Super",
        }
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Modifier {
    type Err = KeyParseError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        [
            Modifier::Alt,
            Modifier::Control,
            Modifier::Shift,
            Modifier::Super,
        ]
        .into_iter()
        .find(|modifier| modifier.name() == name)
        .ok_or_else(|| KeyParseError::UnknownModifier(name.to_string()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeyParseError {
    /// The name is neither a single character nor an xkb keysym name.
    UnknownSymbol(String),
    /// The name is not one of the modifier names.
    UnknownModifier(String),
}

impl fmt::Display for KeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyParseError::UnknownSymbol(name) => {
                write!(f, "{name:?} is not an xkb keysym name")
            }
            KeyParseError::UnknownModifier(name) => {
                write!(f, "{name:?} is not a modifier name")
            }
        }
    }
}

impl std::error::Error for KeyParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use xkb::Keysym;

    #[test]
    fn a_single_character_parses_to_the_symbol_that_types_it() {
        assert_eq!("a".parse(), Ok(Symbol(Keysym::a)));
        assert_eq!("A".parse(), Ok(Symbol(Keysym::A)));
        assert_eq!("!".parse(), Ok(Symbol(Keysym::exclam)));
        assert_eq!("ä".parse(), Ok(Symbol(Keysym::adiaeresis)));
        assert_eq!(" ".parse(), Ok(Symbol(Keysym::space)));
    }

    #[test]
    fn an_xkb_keysym_name_parses_exactly() {
        assert_eq!("Escape".parse(), Ok(Symbol::ESCAPE));
        assert_eq!("Caps_Lock".parse(), Ok(Symbol(Keysym::Caps_Lock)));
        assert_eq!("space".parse(), Ok(Symbol(Keysym::space)));
        assert_eq!("XF86AudioMute".parse(), Ok(Symbol(Keysym::XF86_AudioMute)));
    }

    #[test]
    fn every_xkb_spelling_of_a_symbol_is_accepted() {
        assert_eq!("Page_Up".parse::<Symbol>(), "Prior".parse::<Symbol>());
    }

    #[test]
    fn a_name_xkb_does_not_know_is_rejected() {
        for name in ["capslock", "escape", "Caps Lock", ""] {
            assert_eq!(
                name.parse::<Symbol>(),
                Err(KeyParseError::UnknownSymbol(name.to_string()))
            );
        }
    }

    #[test]
    fn a_modifier_parses_by_its_exact_name() {
        assert_eq!("Shift".parse(), Ok(Modifier::Shift));
        assert_eq!(
            "shift".parse::<Modifier>(),
            Err(KeyParseError::UnknownModifier("shift".to_string()))
        );
    }

    #[test]
    fn a_symbol_that_types_a_visible_character_renders_as_that_character() {
        assert_eq!(Symbol(Keysym::a).to_string(), "a");
        assert_eq!(Symbol(Keysym::A).to_string(), "A");
        assert_eq!(Symbol(Keysym::exclam).to_string(), "!");
        assert_eq!(Symbol(Keysym::KP_1).to_string(), "1");
    }

    #[test]
    fn other_symbols_render_by_their_canonical_xkb_name() {
        assert_eq!(Symbol(Keysym::space).to_string(), "space");
        assert_eq!(Symbol(Keysym::Tab).to_string(), "Tab");
        assert_eq!(Symbol(Keysym::Caps_Lock).to_string(), "Caps_Lock");
        assert_eq!(Symbol(Keysym::Page_Up).to_string(), "Prior");
    }

    #[test]
    fn a_symbol_types_its_character_unless_it_is_a_control_character() {
        assert_eq!(Symbol(Keysym::a).character(), Some('a'));
        assert_eq!(Symbol(Keysym::space).character(), Some(' '));
        assert_eq!(Symbol::RETURN.character(), None);
        assert_eq!(Symbol::ESCAPE.character(), None);
    }

    #[test]
    fn a_key_renders_its_modifiers_before_the_symbol() {
        let mut key = Key::new(Symbol(Keysym::Up));
        key.add_modifier(Modifier::Shift);
        key.add_modifier(Modifier::Control);
        assert_eq!(key.to_string(), "Control+Shift+Up");
    }
}
