use crate::screen::error::ErrorScreen;
use crate::screen::layer_navigation::LayerNavigationScreen;
use crate::screen::parameter_input::ParameterInputScreen;
use crate::screen::Screen;
use crate::service::GuiService;
use longcut_core::port::view::{View, ViewModel};

pub struct GuiView<'a> {
    gui: &'a GuiService<'a>,
}

impl<'a> GuiView<'a> {
    pub fn new(gui: &'a GuiService) -> Self {
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

        self.gui.display_screen(screen);
    }
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
