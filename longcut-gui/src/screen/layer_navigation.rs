use crate::component::action::Action;
use crate::component::layer_stack::LayerStack;
use crate::theme::Theme;
use longcut_core::port::view::LayerNavigationViewModel;
use longcut_graphics_lib::component::column::Column;
use longcut_graphics_lib::component::root::Root;
use longcut_graphics_lib::component::table::Table;
use longcut_graphics_lib::component::Component;

#[derive(Debug)]
pub struct LayerNavigationScreen {
    pub stack: Vec<String>,
    pub actions: Vec<Action>,
}

impl LayerNavigationScreen {
    pub fn assemble(&self, theme: &Theme) -> Box<dyn Component> {
        let layer_stack = LayerStack::new(&self.stack).assemble();

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
