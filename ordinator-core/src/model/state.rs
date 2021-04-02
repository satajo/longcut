use crate::model::effect::Effect;
use crate::model::key::KeyPress;
use crate::model::layer::Layer;

pub struct State {
    root_layer: Layer,
    launch_keys: Vec<KeyPress>,
    end_keys: Vec<KeyPress>,
    active_layer: Option<Layer>,
}

impl State {
    pub fn new(root_layer: Layer, launch_keys: Vec<KeyPress>, end_keys: Vec<KeyPress>) -> State {
        return State {
            active_layer: None,
            root_layer,
            launch_keys,
            end_keys,
        };
    }

    pub fn get_launch_keys(&self) -> &Vec<KeyPress> {
        &self.launch_keys
    }

    pub fn is_active(&self) -> bool {
        self.active_layer.is_some()
    }

    pub fn begin_sequence(&mut self) {
        self.active_layer = Some(self.root_layer.clone());
    }

    pub fn end_sequence(&mut self) {
        self.active_layer = None;
    }

    pub fn set_active_layer(&mut self, layer: Layer) {
        self.active_layer = Some(layer);
    }

    pub fn get_active_layer(&self) -> &Option<Layer> {
        &self.active_layer
    }

    pub fn handle_keypress(&self, press: &KeyPress) -> Vec<Effect> {
        match &self.active_layer {
            None => vec![],
            Some(layer) => {
                if self.end_keys.contains(press) {
                    vec![Effect::End()]
                } else {
                    match layer.actions.get(press) {
                        None => vec![Effect::NotFound()],
                        Some(effects) => effects.clone(),
                    }
                }
            }
        }
    }
}
