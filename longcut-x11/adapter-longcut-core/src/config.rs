use longcut_core::SessionMode;
use longcut_core::config::{KeySchema, OneOrManySchema};
use longcut_core::model::key::Key;
use serde::Deserialize;

/// The x11 configuration section.
#[derive(Debug, Deserialize)]
#[serde(try_from = "ConfigSchema")]
pub struct Config {
    /// The keys that start a session, with the session each one starts.
    pub launch_keys: Vec<(Key, SessionMode)>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigSchema {
    launcher: LauncherSchema,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LauncherSchema {
    keys_launch_global: Option<OneOrManySchema<KeySchema>>,
    keys_launch_app: Option<OneOrManySchema<KeySchema>>,
}

impl TryFrom<ConfigSchema> for Config {
    type Error = String;

    fn try_from(value: ConfigSchema) -> Result<Self, Self::Error> {
        let keys_launch_global: Vec<Key> = match value.launcher.keys_launch_global {
            None => vec![],
            Some(keys) => keys.try_into()?,
        };
        let keys_launch_app: Vec<Key> = match value.launcher.keys_launch_app {
            None => vec![],
            Some(keys) => keys.try_into()?,
        };

        if let Some(key) = keys_launch_global
            .iter()
            .find(|key| keys_launch_app.contains(key))
        {
            return Err(format!(
                "Key {key} is in both keys_launch_global and keys_launch_app, so it cannot tell which \
                 session to launch"
            ));
        }
        if keys_launch_global.is_empty() && keys_launch_app.is_empty() {
            return Err(
                "At least one of keys_launch_global and keys_launch_app is required".into(),
            );
        }

        let global = keys_launch_global
            .into_iter()
            .map(|key| (key, SessionMode::Global));
        let window = keys_launch_app
            .into_iter()
            .map(|key| (key, SessionMode::Window));
        Ok(Self {
            launch_keys: global.chain(window).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> Result<Config, String> {
        serde_norway::from_str::<Config>(yaml).map_err(|error| error.to_string())
    }

    #[test]
    fn launch_keys_carry_the_session_they_launch() {
        let config = parse(
            "launcher:\n  keys_launch_global: [Super_L, Super_R]\n  keys_launch_app: {key: w, modifiers: Super}\n",
        )
        .unwrap();

        let keys: Vec<(String, SessionMode)> = config
            .launch_keys
            .iter()
            .map(|(key, mode)| (key.to_string(), *mode))
            .collect();
        assert_eq!(
            keys,
            vec![
                ("Super_L".to_string(), SessionMode::Global),
                ("Super_R".to_string(), SessionMode::Global),
                ("Super+w".to_string(), SessionMode::Window),
            ]
        );
    }

    #[test]
    fn either_kind_of_launch_key_suffices() {
        assert!(parse("launcher:\n  keys_launch_global: Super_L\n").is_ok());
        assert!(parse("launcher:\n  keys_launch_app: Super_L\n").is_ok());
    }

    #[test]
    fn no_launch_keys_at_all_is_an_error() {
        let error = parse("launcher: {}\n").unwrap_err();
        assert!(
            error.contains("At least one of keys_launch_global and keys_launch_app is required")
        );
    }

    #[test]
    fn a_key_cannot_launch_both_sessions() {
        let error =
            parse("launcher:\n  keys_launch_global: Super_L\n  keys_launch_app: [w, Super_L]\n")
                .unwrap_err();
        assert!(error.contains("Key Super_L is in both keys_launch_global and keys_launch_app"));
    }

    #[test]
    fn a_key_that_is_not_an_xkb_keysym_is_reported_by_name() {
        let error = parse("launcher:\n  keys_launch_global: capslock\n").unwrap_err();
        assert!(error.contains("\"capslock\" is not an xkb keysym name"));
    }

    #[test]
    fn an_unknown_launcher_field_is_an_error() {
        let error = parse("launcher:\n  keys_launch_global: Super_L\n  keys_activate: Super_R\n")
            .unwrap_err();
        assert!(error.contains("keys_activate"));
    }
}
