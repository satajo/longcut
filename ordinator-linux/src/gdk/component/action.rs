use crate::gdk::component::shortcut::Shortcut;
use crate::gdk::config::Theme;
use ordinator_core::model::key::Key;
use ordinator_core::port::view::ViewAction;
use ordinator_gui::component::row::Row;
use ordinator_gui::component::text::Text;
use ordinator_gui::property::Property;
use ordinator_gui::Component;

#[derive(Debug)]
pub struct Action {
    pub shortcut: Shortcut,
    pub name: String,
    pub kind: ActionKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ActionKind {
    Branch,
    Execute,
    System,
}

impl Action {
    pub fn new(key: &Key, action: &ViewAction) -> Self {
        let (name, kind) = match action {
            ViewAction::Branch(layer) => (layer.clone(), ActionKind::Branch),
            ViewAction::Execute(command) => (command.clone(), ActionKind::Execute),
            ViewAction::Unbranch => ("Unbranch".to_string(), ActionKind::System),
            ViewAction::Deactivate => ("Deactivate".to_string(), ActionKind::System),
            ViewAction::Retry => ("Retry".to_string(), ActionKind::System),
        };

        Self {
            shortcut: Shortcut::new(key),
            name,
            kind,
        }
    }

    pub fn assemble(&self, theme: &Theme) -> impl Component {
        let shortcut = self.shortcut.assemble().width(125);
        let name_text = Text::new(self.name.to_string());
        let color = match self.kind {
            ActionKind::Branch => theme.action_branch_color.clone(),
            ActionKind::Execute => theme.action_execute_color.clone(),
            ActionKind::System => theme.action_system_color.clone(),
        };

        Row::<Box<dyn Component>>::new()
            .add_child(Box::new(shortcut))
            .add_child(Box::new(name_text))
            .gap_size(8)
            .foreground(color)
    }
}
