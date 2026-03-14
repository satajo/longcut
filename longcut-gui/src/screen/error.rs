use crate::component::action::Action;
use crate::component::root::Root;
use crate::model::theme::Theme;
use itertools::Itertools;
use longcut_core::port::view::ErrorViewModel;
use longcut_graphics_lib::component::Component;
use longcut_graphics_lib::component::column::Column;
use longcut_graphics_lib::component::table::Table;
use longcut_graphics_lib::component::text::Text;
use longcut_graphics_lib::model::unit::Unit;

#[derive(Debug)]
pub struct ErrorScreen {
    pub actions: Vec<Action>,
    pub error_details: String,
    pub error_type: String,
}

impl ErrorScreen {
    #[must_use]
    pub fn assemble(&self, theme: &Theme) -> Box<dyn Component> {
        let error_type = Text::new(self.error_type.to_uppercase());

        let mut error_details: Column<Text> = Column::new();
        for error_detail in self.error_details.lines() {
            error_details = error_details.add_child(Text::new(error_detail.to_string()));
        }

        let mut actions = Table::new(400);
        for action in &self.actions {
            actions = actions.add_child(action.assemble(theme));
        }

        let column = Column::<Box<dyn Component>>::new()
            .add_child(Box::new(error_type))
            .add_child(Box::new(error_details))
            .add_child(Box::new(actions))
            .gap_size(Unit::Em(1.0));

        let root = Root::new(
            theme.error_background_color.clone(),
            theme.error_foreground_color.clone(),
            theme.font.clone(),
            theme.error_border_color.clone(),
            column,
        );

        Box::new(root)
    }
}

impl From<ErrorViewModel<'_>> for ErrorScreen {
    fn from(data: ErrorViewModel) -> Self {
        let actions = data
            .actions
            .iter()
            .map(|(key, action)| Action::new(key, action))
            .sorted()
            .collect();

        Self {
            actions,
            error_details: data.error_details.to_string(),
            error_type: data.error_type.to_string(),
        }
    }
}
