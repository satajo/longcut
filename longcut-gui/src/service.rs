use crate::config::Config;
use crate::model::theme::Theme;
use crate::model::window_properties::WindowProperties;
use crate::port::window_manager::WindowManager;
use crate::screen::Screen;
use longcut_graphics_lib::render_component;

pub struct GuiService<'a> {
    window_manager: &'a dyn WindowManager,
    theme: Theme,
    window_properties: WindowProperties,
}

impl<'a> GuiService<'a> {
    pub fn new(window_manager: &'a dyn WindowManager, config: Config) -> Self {
        Self {
            window_manager,
            theme: config.theme,
            window_properties: config.window_properties,
        }
    }

    pub fn display_screen(&self, screen: Screen) {
        let theme = self.theme.clone();
        let window_props = self.window_properties.clone();
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
