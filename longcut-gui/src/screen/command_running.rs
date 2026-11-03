use crate::component::action::Action;
use crate::component::root::Root;
use crate::model::theme::Theme;
use itertools::Itertools;
use longcut_core::port::view::CommandRunningViewModel;
use longcut_graphics_lib::component::Component;
use longcut_graphics_lib::component::column::Column;
use longcut_graphics_lib::component::table::Table;
use longcut_graphics_lib::component::text::Text;
use longcut_graphics_lib::model::unit::Unit;

/// A synchronous command that is still running, with the keys that kill it.
#[derive(Debug)]
pub struct CommandRunningScreen {
    actions: Vec<Action>,
    command_name: String,
    program: String,
    elapsed_secs: u64,
}

impl CommandRunningScreen {
    #[must_use]
    pub fn assemble(&self, theme: &Theme) -> Box<dyn Component> {
        let title = Text::new(format!("RUNNING  {}", self.command_name));

        let mut program: Column<Text> = Column::new();
        for line in self.program.lines() {
            program = program.add_child(Text::new(line.to_string()));
        }

        let elapsed = Text::new(format!("{} s", self.elapsed_secs));

        let mut actions = Table::new(Action::COLUMN_WIDTH);
        for action in &self.actions {
            actions = actions.add_child(action.assemble(theme));
        }

        let column = Column::<Box<dyn Component>>::new()
            .add_child(Box::new(title))
            .add_child(Box::new(program))
            .add_child(Box::new(elapsed))
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

impl From<CommandRunningViewModel<'_>> for CommandRunningScreen {
    fn from(data: CommandRunningViewModel) -> Self {
        let actions = data
            .actions
            .iter()
            .map(|(key, action)| Action::new(key, action))
            .sorted()
            .collect();

        Self {
            actions,
            command_name: data.command_name.to_string(),
            program: data.program.to_string(),
            elapsed_secs: data.elapsed.as_secs(),
        }
    }
}
