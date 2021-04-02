use itertools::Itertools;
use ordinator_core::model::effect::Effect;
use ordinator_core::model::key::KeyPress;
use ordinator_core::model::layer::Layer;
use ordinator_core::model::state::State;

pub struct Settings {
    pub padding: u16,
}

pub struct Continuation {
    pub shortcut: String,
    pub name: String,
}

pub struct ViewModel {
    pub visible: bool,
    pub sequence: Vec<Continuation>,
    pub continuations: Vec<Continuation>,
    pub settings: Settings,
}

fn describe_keypress(keypress: &KeyPress) -> String {
    keypress.code.to_string()
}

fn describe_effect(effect: &Effect) -> String {
    match effect {
        Effect::Branch(layer) => {
            format!("Select layer {}", layer.name)
        }
        Effect::End() => "End".to_string(),
        Effect::Execute(name) => format!("Run command {}", name).to_string(),
        Effect::NotFound() => "Not found".to_string(),
    }
}

fn describe_effects(effects: &Vec<Effect>) -> String {
    effects.into_iter().map(describe_effect).join(", ")
}

impl ViewModel {
    pub fn empty() -> Self {
        return ViewModel {
            visible: false,
            sequence: Vec::new(),
            continuations: Vec::new(),
            settings: Settings { padding: 8 },
        };
    }

    pub fn from_model(model: &State) -> Self {
        let mut vm = Self::empty();
        match model.get_active_layer() {
            None => {
                vm.visible = false;
            }
            Some(layer) => {
                vm.visible = true;
                for (keypress, effects) in &layer.actions {
                    vm.continuations.push(Continuation {
                        shortcut: describe_keypress(keypress),
                        name: describe_effects(effects),
                    });
                }
            }
        }
        return vm;
    }
}
