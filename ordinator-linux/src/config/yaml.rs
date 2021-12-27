use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Shortcut {
    Key(String),
    KeyAndModifiers {
        key: String,
        modifiers: OneOrMany<String>,
    },
}

pub type Step = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub shortcut: Shortcut,
    pub steps: OneOrMany<Step>,

    #[serde(rename = "final")]
    #[serde(default = "default_true")]
    pub is_final: bool,

    #[serde(rename = "synchronous")]
    #[serde(default = "default_true")]
    pub is_synchronous: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Layer {
    pub commands: Option<Vec<Command>>,
    pub layers: Option<Vec<Layer>>,
    pub name: String,
    pub shortcut: Shortcut,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RootLayer {
    pub commands: Option<Vec<Command>>,
    pub layers: Option<Vec<Layer>>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Keys {
    pub activate: OneOrMany<Shortcut>,
    pub back: Option<OneOrMany<Shortcut>>,
    pub deactivate: Option<OneOrMany<Shortcut>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct View {
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConfiguration {
    pub root: RootLayer,
    pub keys: Keys,
    pub view: View,
}

impl YamlConfiguration {
    pub fn parse(file: &File) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_reader(file)
    }
}

// Serde workaround for boolean default values

fn default_true() -> bool {
    true
}
