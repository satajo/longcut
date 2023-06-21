use crate::module::X11Module;
use longcut_core::model::key::{Key, Modifier, Symbol};
use longcut_core::port::input::Input;
use x11::xlib::{ControlMask, Mod1Mask, Mod4Mask, ShiftMask};

pub struct X11Input<'a> {
    x11: &'a X11Module,
}

impl<'a> X11Input<'a> {
    pub fn new(x11: &'a X11Module) -> Self {
        Self { x11 }
    }

    /// Loops on reading x11 key press events until the first one which is a valid key.
    fn await_for_input(&self) -> Key {
        loop {
            let event = self.x11.handle.read_next_keypress();

            let grapheme = self.x11.handle.keypress_to_grapheme(&event);
            let key_name = self.x11.handle.keypress_to_key_name(&event);
            let parsed_symbol = match (key_name, grapheme) {
                (None, None) => continue,
                (Some(k), None) => x11_name_to_symbol(k.as_str()),
                (None, Some(g)) => x11_name_to_symbol(g.as_str()),
                (Some(k), Some(g)) => {
                    let ksym = x11_name_to_symbol(k.as_str());
                    let gsym = x11_name_to_symbol(g.as_str());

                    if let Ok(ksymbol) = &ksym {
                        if let Symbol::Character(_) = &ksymbol {
                            // If the key name maps into a single character representation, a character
                            // was typed -> return the grapheme instead.
                            gsym
                        } else {
                            // The key name maps into a special character -> return the special char.
                            ksym
                        }
                    } else {
                        // Key name mapping failed, return the grapheme.
                        gsym
                    }
                }
            };

            let mut press = if let Ok(symbol) = parsed_symbol {
                Key::new(symbol)
            } else {
                println!("{event:?} was not a valid symbol!");
                continue;
            };

            // Active modifier states are added to the key press.
            if event.is_mod_active(ShiftMask) {
                press.add_modifier(Modifier::Shift);
            }

            if event.is_mod_active(ControlMask) {
                press.add_modifier(Modifier::Control);
            }

            if event.is_mod_active(Mod1Mask) {
                press.add_modifier(Modifier::Alt);
            }

            if event.is_mod_active(Mod4Mask) {
                press.add_modifier(Modifier::Super);
            }

            return press;
        }
    }

    fn keys_to_x11_keycodes(&self, keys: &[Key]) -> Vec<u8> {
        keys.iter()
            .map(|key| symbol_to_x11_name(&key.symbol))
            .filter_map(|sym| self.x11.handle.string_to_keycode(&sym))
            .collect()
    }
}

impl<'a> Input for X11Input<'a> {
    fn capture_one(&self, keys: &[Key]) -> Key {
        let keycodes: Vec<u8> = self.keys_to_x11_keycodes(keys);
        self.x11.handle.grab_keys(keycodes.clone());
        let key = self.await_for_input();
        self.x11.handle.free_keys(keycodes);
        key
    }

    fn capture_any(&self) -> Key {
        self.x11.handle.grab_keyboard();
        let press = self.await_for_input();
        self.x11.handle.free_keyboard();
        press
    }
}

fn symbol_to_x11_name(symbol: &Symbol) -> String {
    match symbol {
        Symbol::AltL => "Alt_L".to_string(),
        Symbol::AltR => "Alt_R".to_string(),
        Symbol::PageDown => "Next".to_string(),
        Symbol::PageUp => "Prior".to_string(),
        Symbol::ShiftL => "Shift_L".to_string(),
        Symbol::ShiftR => "Shift_R".to_string(),
        Symbol::SuperL => "Super_L".to_string(),
        Symbol::SuperR => "Super_R".to_string(),
        Symbol::Character(c) => c.to_string(),
        otherwise => format!("{:?}", otherwise),
    }
}

fn x11_name_to_symbol(name: &str) -> Result<Symbol, ()> {
    let symbol = match name {
        "Alt_L" => Symbol::AltL,
        "Alt_R" => Symbol::AltR,
        "Prior" => Symbol::PageUp,
        "Next" => Symbol::PageDown,
        "Shift_L" => Symbol::ShiftL,
        "Shift_R" => Symbol::ShiftR,
        "Super_L" => Symbol::SuperL,
        "Super_R" => Symbol::SuperR,
        otherwise => Symbol::try_from(otherwise).map_err(|_| ())?,
    };

    Ok(symbol)
}
