use crate::gdk::config::{Alignment, Config, Dimensions, Position, WindowConfig};
use gdk::cairo;

pub struct Window<'a> {
    config: &'a Config,
    gdk_window: gdk::Window,
}

impl<'a> Window<'a> {
    pub fn new(config: &'a Config) -> Self {
        let position = position_window(&config.window);
        let gdk_window = gdk::Window::new(
            None,
            &gdk::WindowAttr {
                title: None,
                event_mask: gdk::EventMask::empty(),
                x: Some(position.x),
                y: Some(position.y),
                width: config.window.size.horizontal as i32,
                height: config.window.size.vertical as i32,
                wclass: gdk::WindowWindowClass::InputOutput,
                visual: None,
                window_type: gdk::WindowType::Toplevel,
                cursor: None,
                override_redirect: false,
                type_hint: Some(gdk::WindowTypeHint::Dock),
            },
        );

        gdk_window.set_keep_above(true);
        gdk_window.set_override_redirect(true);

        Self { config, gdk_window }
    }

    pub fn show(&self, f: impl FnOnce(cairo::Context)) {
        self.gdk_window.show();

        let region = self.gdk_window.visible_region().unwrap();
        let drawing_context = self.gdk_window.begin_draw_frame(&region).unwrap();
        let cairo_context = drawing_context.cairo_context().unwrap();

        // Screen is blanked before beginning draw.
        cairo_context.set_source_rgb(0.0, 0.0, 0.0);
        cairo_context.paint().expect("Cairo Context error");

        f(cairo_context);

        self.gdk_window.end_draw_frame(&drawing_context);
        gdk::flush();
    }

    pub fn hide(&self) {
        if self.gdk_window.is_visible() {
            self.gdk_window.hide();
            gdk::flush();
        }
    }
}

fn position_window(config: &WindowConfig) -> Position {
    let align_position = |alignment: &Alignment, width: u32, max_width: u32| -> i32 {
        (match alignment {
            Alignment::Beginning => 0,
            Alignment::Center => (max_width - width) / 2,
            Alignment::End => max_width - width,
        }) as i32
    };

    let screen_dimensions = get_screen_geometry();
    Position {
        x: align_position(
            &config.horizontal,
            config.size.horizontal,
            screen_dimensions.horizontal,
        ),
        y: align_position(
            &config.vertical,
            config.size.vertical,
            screen_dimensions.vertical,
        ),
    }
}

fn get_screen_geometry() -> Dimensions {
    let geometry = gdk::Display::default()
        .expect("No default display")
        .primary_monitor()
        .expect("No default monitor")
        .geometry();

    Dimensions {
        vertical: geometry.height as u32,
        horizontal: geometry.width as u32,
    }
}
