use crate::component::action::Action;
use crate::component::column::Column;
use crate::component::root::Root;
use crate::component::table::Table;
use crate::component::Component;
use crate::model::theme::Theme;
use crate::screen::render_layer_stack;
use longcut_core::port::view::LayerNavigationViewModel;

#[derive(Debug)]
pub struct LayerNavigationScreen {
    pub stack: Vec<String>,
    pub actions: Vec<Action>,
}

impl LayerNavigationScreen {
    pub fn assemble(&self, theme: &Theme) -> Box<dyn Component + 'static> {
        let layer_stack = render_layer_stack(&self.stack);

        let mut actions = Table::new(400);
        for action in &self.actions {
            actions = actions.add_child(action.assemble(theme));
        }

        let column = Column::<Box<dyn Component>>::new()
            .add_child(Box::new(layer_stack))
            .add_child(Box::new(actions))
            .gap_size(20);

        let root = Root::new(
            theme.background_color.clone(),
            theme.foreground_color.clone(),
            theme.border_color.clone(),
            column,
        );

        Box::new(root)
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
