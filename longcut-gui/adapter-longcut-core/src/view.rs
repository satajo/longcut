use longcut_core::port::view::{View, ViewModel};
use longcut_gui::ErrorScreen;
use longcut_gui::GuiService;
use longcut_gui::LayerNavigationScreen;
use longcut_gui::ParameterInputScreen;
use longcut_gui::Screen;

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
        match screen_for_view_model(model) {
            Some(screen) => self.gui.display_screen(screen),
            None => self.gui.hide_gui(),
        }
    }
}

fn screen_for_view_model(view_model: ViewModel) -> Option<Screen> {
    match view_model {
        ViewModel::None => None,
        ViewModel::Error(model) => {
            let screen = ErrorScreen::from(model);
            Some(Screen::Error(screen))
        }
        ViewModel::LayerNavigation(model) => {
            let screen = LayerNavigationScreen::from(model);
            Some(Screen::LayerNavigation(screen))
        }
        ViewModel::ParameterInput(model) => {
            let screen = ParameterInputScreen::from(model);
            Some(Screen::ParameterInput(screen))
        }
    }
}
