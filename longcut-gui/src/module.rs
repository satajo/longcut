use crate::config::Config;
use crate::port::window_manager::WindowManager;
use crate::screen::Screen;
use crate::theme::Theme;
use crate::window_properties::WindowProperties;
use longcut_config::{ConfigError, ConfigModule, Module};
use longcut_graphics_lib::render_component;

pub struct GuiModule<'a> {
    window_manager: &'a dyn WindowManager,
    theme: Theme,
    window_properties: WindowProperties,
}

impl Module for GuiModule<'_> {
    const IDENTIFIER: &'static str = "gui";

    type Config = Config;
}

impl<'a> GuiModule<'a> {
    pub fn new(
        config_module: &'a ConfigModule,
        window_manager: &'a dyn WindowManager,
    ) -> Result<Self, ConfigError> {
        let config = config_module.config_for_module::<Self>()?;

        Ok(Self {
            window_manager,
            theme: config.theme,
            window_properties: config.window_properties,
        })
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
