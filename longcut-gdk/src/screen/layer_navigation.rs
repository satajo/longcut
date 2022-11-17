use crate::component;
use crate::component::action::Action;
use crate::config::Theme;
use longcut_core::port::view::LayerNavigationViewModel;
use longcut_gui::component::column::Column;
use longcut_gui::component::table::Table;
use longcut_gui::Component;

#[derive(Debug)]
pub struct LayerNavigationScreen {
    pub stack: Vec<String>,
    pub actions: Vec<Action>,
}

impl LayerNavigationScreen {
    pub fn assemble(&self, theme: &Theme) -> impl Component {
        let layer_stack = component::render_layer_stack(&self.stack);

        let mut actions = Table::new(400);
        for action in &self.actions {
            actions = actions.add_child(action.assemble(theme));
        }

        let column = Column::<Box<dyn Component>>::new()
            .add_child(Box::new(layer_stack))
            .add_child(Box::new(actions))
            .gap_size(20);

        component::view_root(
            theme.background_color.clone(),
            theme.foreground_color.clone(),
            theme.border_color.clone(),
            column,
        )
    }
}

impl From<LayerNavigationViewModel<'_>> for LayerNavigationScreen {
    fn from(data: LayerNavigationViewModel) -> Self {
        let stack = data.layers.iter().map(|layer| layer.name.clone()).collect();
        let actions = data
            .actions
            .iter()
            .map(|(key, action)| Action::new(key, action))
            .collect();

        Self { stack, actions }
    }
}
