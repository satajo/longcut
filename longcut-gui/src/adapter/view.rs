use crate::screen::error::ErrorScreen;
use crate::screen::layer_navigation::LayerNavigationScreen;
use crate::screen::parameter_input::ParameterInputScreen;
use crate::{Component, GuiModule};
use longcut_core::port::view::{View, ViewModel};

pub struct GuiView<'a> {
    gui: &'a GuiModule<'a>,
}

impl<'a> GuiView<'a> {
    pub fn new(gui: &'a GuiModule) -> Self {
        Self { gui }
    }
}

impl<'a> View for GuiView<'a> {
    fn render(&self, model: ViewModel) {
        // TODO: Make this more obvious by separating the hidden vs visible state in ViewModel already.
        let Ok(screen) = Screen::try_from(model) else {
            // No displayable screen exists so we hide the gui.
            self.gui.hide_gui();
            return;
        };

        let theme = self.gui.config.theme.clone();
        let assembly_fn: Box<dyn FnOnce() -> Box<dyn Component + 'static> + Send> =
            Box::new(move || match screen {
                Screen::LayerNavigation(screen) => Box::new(screen.assemble(&theme)),
                Screen::ParameterInput(screen) => Box::new(screen.assemble(&theme)),
                Screen::Error(screen) => Box::new(screen.assemble(&theme)),
            });

        self.gui.display_gui(assembly_fn);
    }
}

enum Screen {
    LayerNavigation(LayerNavigationScreen),
    ParameterInput(ParameterInputScreen),
    Error(ErrorScreen),
}

impl TryFrom<ViewModel<'_>> for Screen {
    type Error = ();

    fn try_from(value: ViewModel) -> Result<Self, <Screen as TryFrom<ViewModel<'_>>>::Error> {
        match value {
            ViewModel::None => Err(()),
            ViewModel::Error(model) => {
                let screen = ErrorScreen::from(model);
                Ok(Self::Error(screen))
            }
            ViewModel::LayerNavigation(model) => {
                let screen = LayerNavigationScreen::from(model);
                Ok(Self::LayerNavigation(screen))
            }
            ViewModel::ParameterInput(model) => {
                let screen = ParameterInputScreen::from(model);
                Ok(Self::ParameterInput(screen))
            }
        }
    }
}
