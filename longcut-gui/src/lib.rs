use crate::config::Config;
use crate::port::window_manager::WindowManager;
use crate::screen::Screen;
use longcut_graphics_lib::render_component;

pub mod adapter;
mod component;
pub mod config;
pub mod port;
mod screen;
mod theme;
pub mod window_properties;

pub struct GuiModule<'a> {
    window_manager: &'a dyn WindowManager,
    config: Config,
}

impl<'a> GuiModule<'a> {
    pub fn new(window_manager: &'a dyn WindowManager, config: Config) -> Self {
        Self {
            window_manager,
            config,
        }
    }

    pub fn display_screen(&self, screen: Screen) {
        let theme = self.config.theme.clone();
        let window_props = self.config.window.clone();
        self.window_manager.show_window(
            window_props,
            Box::new(move |dimensions, renderer| {
                let component = match screen {
                    Screen::LayerNavigation(screen) => screen.assemble(&theme),
                    Screen::ParameterInput(screen) => screen.assemble(&theme),
                    Screen::Error(screen) => screen.assemble(&theme),
                };

                render_component(renderer, dimensions, component)
            }),
        );
    }

    pub fn hide_gui(&self) {
        self.window_manager.hide_window();
    }
}
