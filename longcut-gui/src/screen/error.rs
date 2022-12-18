use crate::component::action::Action;
use crate::component::column::Column;
use crate::component::root::Root;
use crate::component::table::Table;
use crate::component::text::Text;
use crate::component::Component;
use crate::model::theme::Theme;
use longcut_core::port::executor::ExecutorError;
use longcut_core::port::view::ErrorViewModel;

#[derive(Debug)]
pub struct ErrorScreen {
    pub actions: Vec<Action>,
    pub error_details: String,
    pub error_type: String,
}

impl ErrorScreen {
    pub fn assemble(&self, theme: &Theme) -> Box<dyn Component + 'static> {
        let error_type = Text::new(format!("{} encountered!", self.error_type));
        let error_details = Text::new(self.error_details.clone());

        let mut actions = Table::new(400);
        for action in &self.actions {
            actions = actions.add_child(action.assemble(theme));
        }

        let column = Column::<Box<dyn Component>>::new()
            .add_child(Box::new(error_type))
            .add_child(Box::new(error_details))
            .add_child(Box::new(actions))
            .gap_size(20);

        let root = Root::new(
            theme.error_background_color.clone(),
            theme.error_foreground_color.clone(),
            theme.error_border_color.clone(),
            column,
        );

        Box::new(root)
    }
}

impl From<ErrorViewModel<'_>> for ErrorScreen {
    fn from(data: ErrorViewModel) -> Self {
        let error_type = match data.error {
            ExecutorError::RuntimeError(_) => "Runtime error".to_string(),
            ExecutorError::StartupError => "Startup error".to_string(),
            ExecutorError::UnknownError => "Unknown error".to_string(),
        };

        let error_details = match data.error {
            ExecutorError::RuntimeError(details) => details.trim().to_string(),
            ExecutorError::StartupError => "Failed to start the target command".to_string(),
            ExecutorError::UnknownError => "No error details available".to_string(),
        };

        let actions = data
            .actions
            .iter()
            .map(|(key, action)| Action::new(key, action))
            .collect();

        Self {
            actions,
            error_details,
            error_type,
        }
    }
}
