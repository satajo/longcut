use crate::component::action::Action;
use crate::component::layer_stack::LayerStack;
use crate::component::root::Root;
use crate::model::theme::Theme;
use itertools::Itertools;
use longcut_core::port::view::LayerNavigationViewModel;
use longcut_graphics_lib::component::column::Column;
use longcut_graphics_lib::component::table::Table;
use longcut_graphics_lib::component::Component;
use longcut_graphics_lib::model::unit::Unit;

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
            .gap_size(Unit::Em(1.0));

        let root = Root::new(
            theme.background_color.clone(),
            theme.foreground_color.clone(),
            theme.font.clone(),
            theme.border_color.clone(),
            column,
        );

        Box::new(root)
    }
}

impl From<LayerNavigationViewModel<'_>> for LayerNavigationScreen {
    fn from(data: LayerNavigationViewModel) -> Self {
        let stack = data
            .layer_stack
            .iter()
            .map(|layer| layer.name.clone())
            .collect();
        let actions = data
            .actions
            .iter()
            .map(|(key, action)| Action::new(key, action))
            .sorted()
            .collect();

        Self { stack, actions }
    }
}
