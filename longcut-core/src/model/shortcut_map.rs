use crate::model::key::{Key, Symbol};
use std::collections::BTreeMap;
use std::ops::Deref;

#[derive(Debug, Default)]
pub struct ShortcutMap<V>(BTreeMap<Key, V>);

impl<V> ShortcutMap<V> {
    #[must_use]
    pub fn new() -> Self {
        ShortcutMap(BTreeMap::new())
    }

    /// # Errors
    ///
    /// Returns the key and value if the shortcut is already assigned.
    pub fn try_assign(&mut self, shortcut: Key, value: V) -> Result<(), (Key, V)> {
        match self.0.get(&shortcut) {
            None => {
                self.0.insert(shortcut, value);
                Ok(())
            }
            Some(_) => Err((shortcut, value)),
        }
    }

    #[must_use]
    pub fn match_exact(&self, shortcut: &Key) -> Option<&V> {
        self.0.get(shortcut)
    }

    /// Automatically assigns mnemonic shortcuts to all the provided values based on their names.
    ///
    /// A value's name corresponds to its "mnemonic formation preference" string, which is used to
    /// find and assign its mnemonic shortcut. Conflicts are resolved by matching on secondary or
    /// later characters in the name, with eventual fall back to set numerical and special characters.
    /// If even this fallback assignment fails, the mnemonic will not be assigned at all, and it is
    /// silently dropped.
    pub fn auto_assign_mnemonics(&mut self, values: Vec<(&str, V)>) {
        const FALLBACK_CHARACTERS: &str = "1234567890,.?/!@#$%&";

        // Remaining assignments is initialized using the provided values.
        let mut remaining_assignments: Vec<(String, V)> = values
            .into_iter()
            .map(|(name, value)| (format!("{name}{FALLBACK_CHARACTERS}"), value))
            .collect();

        // As long as we have assignments remaining, we run assignment batches over and over. The
        // failed assignments in every batch are collected into a failures vector, from where they
        // will be attempted again in the next batch, using a lower priority character for picking
        // the shortcut key.
        for mnemo_key_index in 0.. {
            if remaining_assignments.is_empty() {
                break;
            }
            let mut failed_assignments = vec![];

            for (name, value) in remaining_assignments {
                let Some(mnemo_char) = name.chars().nth(mnemo_key_index) else {
                    continue;
                };

                let Some(symbol) = Symbol::from_char(mnemo_char.to_ascii_lowercase()) else {
                    failed_assignments.push((name, value));
                    continue;
                };
                if let Err((_, value)) = self.try_assign(Key::new(symbol), value) {
                    failed_assignments.push((name, value));
                }
            }

            remaining_assignments = failed_assignments;
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
        let key = Key::new("a".parse().unwrap());
        shortcuts.try_assign(key.clone(), 1).unwrap();
        assert_eq!(shortcuts.get(&key).unwrap(), &1);
    }

    #[test]
    fn double_insert_is_conflict() {
        let mut shortcuts = ShortcutMap::new();
        let key = Key::new("a".parse().unwrap());

        let result_1 = shortcuts.try_assign(key.clone(), 1);
        assert!(result_1.is_ok());

        let result_2 = shortcuts.try_assign(key, 2);
        assert!(result_2.is_err());
    }

    #[test]
    fn a_key_with_extra_modifiers_does_not_match_a_modifierless_shortcut() {
        let mut shortcuts = ShortcutMap::new();
        let key_without_mods = Key::new("a".parse().unwrap());

        let mut key_with_mods = Key::new("a".parse().unwrap());
        key_with_mods.add_modifier(Modifier::Control);

        shortcuts.try_assign(key_without_mods.clone(), 1).unwrap();
        assert!(shortcuts.match_exact(&key_without_mods).is_some());
        assert!(shortcuts.match_exact(&key_with_mods).is_none());
    }

    #[test]
    fn a_key_without_modifiers_does_not_match_a_modified_shortcut() {
        let mut shortcuts = ShortcutMap::new();
        let key_without_mods = Key::new("a".parse().unwrap());

        let mut key_with_mods = Key::new("a".parse().unwrap());
        key_with_mods.add_modifier(Modifier::Control);

        shortcuts.try_assign(key_with_mods.clone(), 1).unwrap();
        assert!(shortcuts.match_exact(&key_without_mods).is_none());
        assert!(shortcuts.match_exact(&key_with_mods).is_some());
    }

    #[test]
    fn auto_assign_with_unique_keys_works() {
        let mut shortcuts = ShortcutMap::new();

        let options: Vec<(&str, u8)> = vec![("alpha", 0), ("beta", 1)];
        shortcuts.auto_assign_mnemonics(options);

        let key_a = Key::new("a".parse().unwrap());
        assert_eq!(shortcuts.match_exact(&key_a).unwrap(), &0);

        let key_b = Key::new("b".parse().unwrap());
        assert_eq!(shortcuts.match_exact(&key_b).unwrap(), &1);

        let key_c = Key::new("c".parse().unwrap());
        assert!(shortcuts.match_exact(&key_c).is_none());
    }

    #[test]
    fn auto_assign_resolves_conflict_by_using_the_next_character() {
        let mut shortcuts = ShortcutMap::new();

        let options: Vec<(&str, u8)> = vec![("alpha", 0), ("apple", 1)];
        shortcuts.auto_assign_mnemonics(options);

        let key_a = Key::new("a".parse().unwrap());
        assert_eq!(shortcuts.match_exact(&key_a).unwrap(), &0);

        let key_b = Key::new("p".parse().unwrap());
        assert_eq!(shortcuts.match_exact(&key_b).unwrap(), &1);
    }

    #[test]
    fn auto_assign_assigns_priority_by_order_of_inputs_when_conflicting() {
        let key_a = Key::new("a".parse().unwrap());

        {
            let mut shortcuts = ShortcutMap::new();
            let alpha_first: Vec<(&str, u8)> = vec![("alpha", 0), ("apple", 1)];
            shortcuts.auto_assign_mnemonics(alpha_first);
            assert_eq!(shortcuts.match_exact(&key_a).unwrap(), &0);
        }

        {
            let mut shortcuts = ShortcutMap::new();
            let apple_first: Vec<(&str, u8)> = vec![("apple", 1), ("alpha", 0)];
            shortcuts.auto_assign_mnemonics(apple_first);
            assert_eq!(shortcuts.match_exact(&key_a).unwrap(), &1);
        }
    }

    #[test]
    fn auto_assign_conflict_resolution_prioritizes_conflict_free_mappings() {
        let mut shortcuts = ShortcutMap::new();

        // Here "abracadabra" has would try to fall back to "b" after being in conflict with "alpha".
        // However, because "banana" maps to "b" conflict-free, it should be preferred.
        let options: Vec<(&str, u8)> = vec![("alpha", 0), ("abracadabra", 1), ("banana", 2)];
        shortcuts.auto_assign_mnemonics(options);

        let key_a = Key::new("a".parse().unwrap());
        assert_eq!(shortcuts.match_exact(&key_a).unwrap(), &0);

        let key_b = Key::new("b".parse().unwrap());
        assert_eq!(shortcuts.match_exact(&key_b).unwrap(), &2);

        let key_r = Key::new("r".parse().unwrap());
        assert_eq!(shortcuts.match_exact(&key_r).unwrap(), &1);
    }

    #[test]
    fn auto_assign_falls_back_to_a_set_of_predefined_characters() {
        let mut shortcuts = ShortcutMap::new();
        let options: Vec<(&str, u8)> = vec![("a", 0), ("a", 1), ("a", 2)];
        shortcuts.auto_assign_mnemonics(options);

        {
            let key = Key::new("a".parse().unwrap());
            assert_eq!(shortcuts.match_exact(&key).unwrap(), &0);
        }

        {
            let key = Key::new("1".parse().unwrap());
            assert_eq!(shortcuts.match_exact(&key).unwrap(), &1);
        }

        {
            let key = Key::new("2".parse().unwrap());
            assert_eq!(shortcuts.match_exact(&key).unwrap(), &2);
        }
    }
}
