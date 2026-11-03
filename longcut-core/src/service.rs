use crate::config::Config;
use crate::logic::{Context, run_layer_navigation_mode, run_window_mode};
use crate::model::key::{Key, Symbol};
use crate::model::session::SessionMode;
use crate::port::input::{InputError, Keyboard};
use crate::port::view::{ErrorViewModel, ViewModel};
use crate::port::{
    Launcher, WindowManager, clock::Clock, executor::Executor, input::Input, view::View,
};
use std::thread::sleep;
use std::time::Duration;

pub struct CoreService<'a> {
    executor: &'a dyn Executor,
    input: &'a dyn Input,
    view: &'a dyn View,
    clock: &'a dyn Clock,
    window_manager: &'a dyn WindowManager,
    config: Config,
}

impl<'a> CoreService<'a> {
    pub fn new(
        executor: &'a dyn Executor,
        input: &'a dyn Input,
        view: &'a dyn View,
        clock: &'a dyn Clock,
        window_manager: &'a dyn WindowManager,
        config: Config,
    ) -> Self {
        Self {
            executor,
            input,
            view,
            clock,
            window_manager,
            config,
        }
    }

    /// Runs one session in the given mode on the taken keyboard, then returns.
    fn run_session(&self, mode: SessionMode, keyboard: &dyn Keyboard) {
        // Command execution retries are always confirmed with Return.
        let keys_retry = [Key::new(Symbol::RETURN)];
        let context = Context {
            executor: self.executor,
            keyboard,
            view: self.view,
            clock: self.clock,
            window_manager: self.window_manager,
            keys_back: &self.config.keys_back,
            keys_exit: &self.config.keys_exit,
            keys_retry: &keys_retry,
            root_layer: &self.config.root_layer,
            app_specific_layers: &self.config.app_specific_layers,
        };
        match mode {
            SessionMode::Global => run_layer_navigation_mode(&context),
            SessionMode::Window => run_window_mode(&context),
        }
        self.view.render(ViewModel::None);
    }

    /// Runs one session per launch, indefinitely. The keyboard is taken for the whole of a
    /// session and given back when the session ends.
    pub fn run_forever(&self, launcher: &dyn Launcher) -> ! {
        loop {
            self.serve_launch(launcher);
        }
    }

    /// Waits for one launch and serves it: takes the keyboard, runs a session in the launched
    /// mode, then gives the keyboard back. Shows why the session could not start when the
    /// keyboard cannot be taken.
    pub(crate) fn serve_launch(&self, launcher: &dyn Launcher) {
        let mode = launcher.wait_for_launch();
        match self.input.take_keyboard() {
            Ok(keyboard) => self.run_session(mode, keyboard.as_ref()),
            Err(error) => self.show_keyboard_unavailable(&error),
        }
    }

    /// Shows why the session could not start, then clears the screen and returns.
    fn show_keyboard_unavailable(&self, error: &InputError) {
        /// How long the reason a session could not start stays on screen. Nothing can dismiss it
        /// earlier, since the keyboard is exactly what could not be had.
        const KEYBOARD_UNAVAILABLE_DISPLAY_TIME: Duration = Duration::from_secs(4);

        self.view.render(ViewModel::Error(ErrorViewModel {
            error_type: "Keyboard unavailable",
            error_details: &error.to_string(),
            actions: &[],
        }));
        sleep(KEYBOARD_UNAVAILABLE_DISPLAY_TIME);
        self.view.render(ViewModel::None);
    }
}

impl std::fmt::Debug for CoreService<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoreService")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use crate::model::command::Command;
    use crate::model::effect::{EffectTemplate, ShellCommandTemplate};
    use crate::model::key::{Key, Symbol};
    use crate::model::layer::Layer;
    use crate::model::session::SessionMode;
    use crate::port::fakes::Event::{ActiveWindow, KeyPressed};
    use crate::port::fakes::{Effect, Scenario, Screen};

    fn key(c: char) -> Key {
        Key::new(Symbol::from_char(c).unwrap())
    }

    fn exit_key() -> Key {
        Key::new(Symbol::ESCAPE)
    }

    fn navigation(layer_stack: &[&str]) -> Effect {
        Effect::Rendered(Screen::LayerNavigation {
            layer_stack: layer_stack.iter().map(ToString::to_string).collect(),
        })
    }

    fn cleared() -> Effect {
        Effect::Rendered(Screen::None)
    }

    #[test]
    fn global_launch_navigates_the_root_layer_on_the_taken_keyboard() {
        Scenario::new()
            .root_layer(Layer::new("Global".to_string()))
            .app_layer("Firefox$", Layer::new("Firefox".to_string()))
            .given(ActiveWindow("Mozilla Firefox".to_string()))
            .launch(SessionMode::Global)
            .effect(Effect::KeyboardTaken)
            .effect(navigation(&["Global"]))
            .then(KeyPressed(exit_key()))
            .effect(cleared())
            .effect(Effect::KeyboardReleased)
            .ends();
    }

    #[test]
    fn window_launch_navigates_the_active_windows_layer_on_the_taken_keyboard() {
        Scenario::new()
            .root_layer(Layer::new("Global".to_string()))
            .app_layer("Firefox$", Layer::new("Firefox".to_string()))
            .given(ActiveWindow("Mozilla Firefox".to_string()))
            .launch(SessionMode::Window)
            .effect(Effect::KeyboardTaken)
            .effect(navigation(&["Firefox"]))
            .then(KeyPressed(exit_key()))
            .effect(cleared())
            .effect(Effect::KeyboardReleased)
            .ends();
    }

    #[test]
    fn final_command_ends_the_session_and_clears_the_screen_before_giving_the_keyboard_back() {
        let mut step = ShellCommandTemplate::new("firefox").unwrap();
        step.set_synchronous(false);
        let mut firefox = Command::new(
            "Firefox".to_string(),
            vec![EffectTemplate::ShellCommand(step)],
            vec![],
        )
        .unwrap();
        firefox.set_final(true);
        let mut root = Layer::new("Global".to_string());
        root.add_command(key('f'), firefox).unwrap();

        Scenario::new()
            .root_layer(root)
            .launch(SessionMode::Global)
            .effect(Effect::KeyboardTaken)
            .effect(navigation(&["Global"]))
            .then(KeyPressed(key('f')))
            .effect(Effect::Background("firefox".to_string()))
            .effect(cleared())
            .effect(Effect::KeyboardReleased)
            .ends();
    }
}
