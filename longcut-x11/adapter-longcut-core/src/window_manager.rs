use longcut_core::port::WindowManager;
use longcut_x11::X11Handle;

pub struct X11WindowManager<'a> {
    handle: &'a X11Handle,
}

impl<'a> X11WindowManager<'a> {
    #[must_use]
    pub fn new(handle: &'a X11Handle) -> Self {
        Self { handle }
    }
}

impl WindowManager for X11WindowManager<'_> {
    fn get_active_window_name(&self) -> Option<String> {
        let window = self.handle.get_active_window()?;

        // WM_CLASS class name is stable per application and the right thing to match against.
        self.handle
            .get_window_class(window)
            .map(|(_instance, class)| class)
    }
}
