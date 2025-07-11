use crate::component::shortcut::Shortcut;
use crate::model::theme::Theme;
use longcut_core::model::key::Key;
use longcut_core::port::view::ViewAction;
use longcut_graphics_lib::component::Component;
use longcut_graphics_lib::component::row::Row;
use longcut_graphics_lib::component::text::Text;
use longcut_graphics_lib::model::unit::Unit;
use longcut_graphics_lib::property::{Foreground, MarginRight, Property};
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq)]
pub struct Action {
    pub shortcut: Shortcut,
    pub name: String,
    pub kind: ActionKind,
}

impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        self.kind
            .cmp(&other.kind)
            .then_with(|| self.shortcut.cmp(&other.shortcut))
    }
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum ActionKind {
    Branch = 1,
    Execute = 2,
    System = 3,
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

    pub fn assemble(&self, theme: &Theme) -> Foreground<Row<MarginRight<Box<dyn Component>>>> {
        let shortcut = self.shortcut.assemble().width(Unit::Em(6.0));
        let name_text = Text::new(self.name.to_string());
        let color = match self.kind {
            ActionKind::Branch => theme.action_branch_color.clone(),
            ActionKind::Execute => theme.action_execute_color.clone(),
            ActionKind::System => theme.action_system_color.clone(),
        };

        Row::<Box<dyn Component>>::new()
            .add_child(Box::new(shortcut))
            .add_child(Box::new(name_text))
            .gap_size(Unit::Em(1.0))
            .foreground(color)
    }
}
