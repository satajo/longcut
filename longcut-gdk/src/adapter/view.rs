use crate::{GdkModule, GuiState};
use longcut_core::port::view::{View, ViewModel};

pub struct GdkView<'a> {
    gdk: &'a GdkModule,
}

impl<'a> GdkView<'a> {
    pub fn new(gdk: &'a GdkModule) -> Self {
        Self { gdk }
    }
}

impl<'a> View for GdkView<'a> {
    fn render(&self, state: ViewModel) {
        self.gdk
            .sender
            .send(GuiState::from(state))
            .expect("Failed to send ViewModel!")
    }
}
