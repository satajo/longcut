use crate::config::Config;
use crate::input::to_active_modifier;
use longcut_config::{ConfigError, ConfigModule, Module};
use longcut_core::SessionMode;
use longcut_core::model::key::Key;
use longcut_core::port::{Launcher, Session};
use longcut_x11::{Hotkey, HotkeyError, Hotkeys, X11Handle};
use std::fmt;

/// Adapts hotkeys bound on the X server into the core [`Launcher`] port.
///
/// Each launch key is bound through a passive grab, so that the X server hands the keyboard
/// to this process from the key's press on. The session that follows reads the keyboard through
/// [`X11Input`](crate::X11Input), and the keyboard is released when the session token is dropped.
pub struct X11Launcher<'a> {
    x11: &'a X11Handle,
    hotkeys: Hotkeys<'a>,
    /// The bound keys in the order the hotkeys were bound, with the session each launches.
    keys: Vec<(Key, SessionMode)>,
}

impl Module for X11Launcher<'_> {
    const IDENTIFIER: &'static str = "x11";

    type Config = Config;
}

/// Why the launcher could not be set up or waited for.
#[derive(Debug)]
pub enum LauncherError {
    /// The configuration section could not be loaded.
    Config(ConfigError),
    /// The launch keys could not be bound or waited for. `key` names the launch key the error
    /// is about, when it is about one.
    Hotkey {
        key: Option<Key>,
        cause: HotkeyError,
    },
}

impl fmt::Display for LauncherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LauncherError::Config(error) => write!(f, "{error}"),
            LauncherError::Hotkey {
                key: Some(key),
                cause,
            } => write!(f, "launch key {key}: {cause}"),
            LauncherError::Hotkey { key: None, cause } => write!(f, "{cause}"),
        }
    }
}

impl std::error::Error for LauncherError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LauncherError::Config(error) => Some(error),
            LauncherError::Hotkey { cause, .. } => Some(cause),
        }
    }
}

impl<'a> X11Launcher<'a> {
    /// Loads the configured launch keys and binds every one of them on the X server.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration section cannot be loaded, or if a key cannot be
    /// bound: no key in the keyboard layout produces it, or another client already binds it.
    pub fn new(config_module: &ConfigModule, x11: &'a X11Handle) -> Result<Self, LauncherError> {
        let config = config_module
            .config_for_module::<Self>()
            .map_err(LauncherError::Config)?;
        let keys = config.launch_keys;
        let hotkeys = keys.iter().map(|(key, _)| to_hotkey(key)).collect();
        let hotkeys = Hotkeys::bind(x11, hotkeys).map_err(|cause| LauncherError::Hotkey {
            key: cause.hotkey().map(|index| keys[index].0.clone()),
            cause,
        })?;
        Ok(Self { x11, hotkeys, keys })
    }
}

impl Launcher for X11Launcher<'_> {
    fn wait_for_launch(&self) -> Box<dyn Session + '_> {
        match self.hotkeys.wait_for_press() {
            Ok(index) => Box::new(X11Session {
                x11: self.x11,
                mode: self.keys[index].1,
            }),
            // Sessions cannot be served without the hotkeys. Process death closes the
            // connection, which releases every grab.
            Err(cause) => {
                let error = LauncherError::Hotkey {
                    key: cause.hotkey().map(|index| self.keys[index].0.clone()),
                    cause,
                };
                panic!("launching is permanently unavailable: {error}");
            }
        }
    }
}

/// A session begun by a hotkey. The keyboard is held for as long as it lives.
struct X11Session<'a> {
    x11: &'a X11Handle,
    mode: SessionMode,
}

impl Session for X11Session<'_> {
    fn mode(&self) -> SessionMode {
        self.mode
    }
}

impl Drop for X11Session<'_> {
    fn drop(&mut self) {
        // A failure here means the connection is gone, which the next wait reports.
        if let Err(error) = self.x11.end_session() {
            eprintln!("could not release the keyboard: {error}");
        }
    }
}

fn to_hotkey(key: &Key) -> Hotkey {
    Hotkey {
        keysym: key.symbol.keysym(),
        modifiers: key
            .modifiers
            .iter()
            .copied()
            .map(to_active_modifier)
            .collect(),
    }
}
