use crate::model::key::Key;

pub enum ViewAction {
    Branch(String),
    Execute(String),
    Unbranch(),
    Deactivate(),
}

pub struct LayerViewData {
    pub actions: Vec<(Key, ViewAction)>,
    pub layers: Vec<String>,
}

pub enum ViewState {
    Hidden,
    LayerView(LayerViewData),
}

pub trait ToViewData {
    fn to_view_data(&self) -> ViewState;
}

pub trait View {
    fn render(&self, state: &ViewState);
}
