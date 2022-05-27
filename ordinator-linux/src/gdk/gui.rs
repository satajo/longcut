use crate::gdk::config::Config;
use crate::gdk::renderer::CairoRenderer;
use crate::gdk::screen::error::ErrorScreen;
use crate::gdk::screen::layer_navigation::LayerNavigationScreen;
use crate::gdk::screen::parameter_input::ParameterInputScreen;
use crate::gdk::window::Window;
use ordinator_core::port::view::ViewModel;
use ordinator_gui::{Component, Context};

pub struct Gui<'a> {
    config: &'a Config,
    window: &'a Window<'a>,
}

impl<'a> Gui<'a> {
    pub fn new(config: &'a Config, window: &'a Window) -> Self {
        Self { config, window }
    }

    pub fn update(&self, model: GuiState) {
        if let GuiState::Hidden = model {
            return self.window.hide();
        }

        self.window.show(|cairo| {
            let renderer = CairoRenderer::new(&cairo).with_font_size(20);
            let initial_color = &self.config.theme.background_color;
            let ctx = Context::new(&renderer, initial_color, self.config.window.size);

            match &model {
                GuiState::Hidden => {
                    // This will not happen, but the case needs to be handled.
                }
                GuiState::Error(model) => model.assemble(&self.config.theme).render(&ctx),
                GuiState::LayerNavigation(model) => model.assemble(&self.config.theme).render(&ctx),
                GuiState::ParameterInput(model) => model.assemble(&self.config.theme).render(&ctx),
            }
        });
    }
}

#[derive(Debug)]
pub enum GuiState {
    Hidden,
    Error(ErrorScreen),
    LayerNavigation(LayerNavigationScreen),
    ParameterInput(ParameterInputScreen),
}

impl<'a> From<ViewModel<'a>> for GuiState {
    fn from(data: ViewModel) -> Self {
        match data {
            ViewModel::None => GuiState::Hidden,
            ViewModel::LayerNavigation(data) => {
                GuiState::LayerNavigation(LayerNavigationScreen::from(data))
            }
            ViewModel::ParameterInput(data) => {
                GuiState::ParameterInput(ParameterInputScreen::from(data))
            }
            ViewModel::Error(data) => GuiState::Error(ErrorScreen::from(data)),
        }
    }
}
