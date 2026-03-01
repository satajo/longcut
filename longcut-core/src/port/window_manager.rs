pub trait WindowManager {
    /// Returns the name of the currently active window, or None if unavailable.
    fn get_active_window_name(&self) -> Option<String>;
}
