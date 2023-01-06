use crate::window_properties::WindowProperties;
use longcut_graphics_lib::model::dimensions::Dimensions;
use longcut_graphics_lib::port::renderer::Renderer;

pub type RenderPassFn = Box<dyn FnOnce(Dimensions, &dyn Renderer) + Send>;

pub trait WindowManager {
    /// Display a Gui window with provided properties.
    ///
    /// The WindowManager then calls the render_fn callback function with the realized window
    /// [Dimensions], which can be different from the ones requested. The render_fn is passed a
    /// graphics_lib compatible [Renderer] implementation, through which the window content can
    /// be rendered.
    ///
    /// The render_fn callback is marked Send to allow communicating with UI systems that run in
    /// a separate thread.
    fn show_window(&self, requested_properties: WindowProperties, render_fn: RenderPassFn);

    /// Instructs the WindowManager to hide the visible Gui window if visible.
    fn hide_window(&self);
}
