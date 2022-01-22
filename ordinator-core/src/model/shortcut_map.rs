use crate::model::key::Key;
use std::collections::BTreeMap;
use std::ops::Deref;

#[derive(Debug, Default)]
pub struct ShortcutMap<V>(BTreeMap<Key, V>);

impl<V> ShortcutMap<V> {
    pub fn new() -> Self {
        ShortcutMap(BTreeMap::new())
    }

    pub fn try_assign(&mut self, shortcut: Key, value: V) -> Result<(), (Key, V)> {
        match self.0.get(&shortcut) {
            None => {
                self.0.insert(shortcut, value);
                Ok(())
            }
            Some(_) => Err((shortcut, value)),
        }
    }

    pub fn match_exact(&self, shortcut: &Key) -> Option<&V> {
        self.0.get(shortcut)
    }

    /// Returns the value matching the shortcut definition or a modifier-less definition if one exists.
    pub fn match_fuzzy(&self, shortcut: &Key) -> Option<&V> {
        match self.match_exact(shortcut) {
            Some(exact) => Some(exact),
            None => self.match_exact(&Key::new(shortcut.symbol.clone())),
        }
    }
}

impl<V> Deref for ShortcutMap<V> {
    type Target = BTreeMap<Key, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::key::Modifier;

    #[test]
    fn can_insert_and_match_inserted() {
        let mut shortcuts = ShortcutMap::new();
        let key = Key::new("a".try_into().unwrap());
        shortcuts.try_assign(key.clone(), 1).unwrap();
        assert_eq!(shortcuts.get(&key).unwrap(), &1);
    }

    #[test]
    fn double_insert_is_conflict() {
        let mut shortcuts = ShortcutMap::new();
        let key = Key::new("a".try_into().unwrap());

        let result_1 = shortcuts.try_assign(key.clone(), 1);
        assert!(result_1.is_ok());

        let result_2 = shortcuts.try_assign(key, 2);
        assert!(result_2.is_err());
    }

    #[test]
    fn exact_match_considers_modifiers() {
        let mut shortcuts = ShortcutMap::new();
        let key_without_mods = Key::new("a".try_into().unwrap());

        let mut key_with_mods = Key::new("a".try_into().unwrap());
        key_with_mods.add_modifier(Modifier::Control);

        shortcuts.try_assign(key_without_mods.clone(), 1).unwrap();
        assert!(shortcuts.match_exact(&key_without_mods).is_some());
        assert!(shortcuts.match_exact(&key_with_mods).is_none());
    }

    #[test]
    fn fuzzy_match_matches_modifier_supersets() {
        let mut shortcuts = ShortcutMap::new();
        let key_without_mods = Key::new("a".try_into().unwrap());

        let mut key_with_mods = Key::new("a".try_into().unwrap());
        key_with_mods.add_modifier(Modifier::Control);

        shortcuts.try_assign(key_without_mods.clone(), 1).unwrap();
        assert!(shortcuts.match_fuzzy(&key_without_mods).is_some());
        assert!(shortcuts.match_fuzzy(&key_with_mods).is_some());
    }

    #[test]
    fn fuzzy_match_ignores_modifier_subsets() {
        let mut shortcuts = ShortcutMap::new();
        let key_without_mods = Key::new("a".try_into().unwrap());

        let mut key_with_mods = Key::new("a".try_into().unwrap());
        key_with_mods.add_modifier(Modifier::Control);

        shortcuts.try_assign(key_with_mods.clone(), 1).unwrap();
        assert!(shortcuts.match_fuzzy(&key_without_mods).is_none());
        assert!(shortcuts.match_fuzzy(&key_with_mods).is_some());
    }

    #[test]
    fn fuzzy_match_only_matches_down_to_modifierless_keys() {
        // This test mostly just documents the current behaviour. In future, forms of "downmatching"
        // the modifiers might be a useful feature!
        let mut shortcuts = ShortcutMap::new();
        let mut key_with_1_mod = Key::new("a".try_into().unwrap());
        key_with_1_mod.add_modifier(Modifier::Control);

        let mut key_with_2_mods = Key::new("a".try_into().unwrap());
        key_with_2_mods.add_modifier(Modifier::Control);
        key_with_2_mods.add_modifier(Modifier::Alt);

        shortcuts.try_assign(key_with_1_mod.clone(), 1).unwrap();
        assert!(shortcuts.match_fuzzy(&key_with_1_mod).is_some());
        assert!(shortcuts.match_fuzzy(&key_with_2_mods).is_none());
    }
}
