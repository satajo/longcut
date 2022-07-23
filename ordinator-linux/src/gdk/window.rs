use gdk::cairo;
use ordinator_gui::model::alignment::Alignment;
use ordinator_gui::model::dimensions::Dimensions;
use ordinator_gui::model::position::Position;

pub struct Config {
    pub size: Dimensions,
    pub horizontal: Alignment,
    pub vertical: Alignment,
}

pub struct Window<'a> {
    config: &'a Config,
    gdk_window: gdk::Window,
}

impl<'a> Window<'a> {
    pub fn new(config: &'a Config) -> Self {
        let (size, position) = calculate_window_geometry(config);
        let gdk_window = gdk::Window::new(
            None,
            &gdk::WindowAttr {
                title: None,
                event_mask: gdk::EventMask::empty(),
                x: Some(position.horizontal as i32),
                y: Some(position.vertical as i32),
                width: size.width as i32,
                height: size.height as i32,
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

fn calculate_window_geometry(config: &Config) -> (Dimensions, Position) {
    let align_position = |alignment: &Alignment, size: u32, max_size: u32| -> i32 {
        (match alignment {
            Alignment::Beginning => 0,
            Alignment::Center => (max_size - size) / 2,
            Alignment::End => max_size - size,
        }) as i32
    };

    let screen_dimensions = get_screen_geometry();
    let window_dimensions = screen_dimensions.intersect(&config.size);

    let window_position = Position {
        horizontal: align_position(
            &config.horizontal,
            window_dimensions.width,
            screen_dimensions.width,
        ) as u32,
        vertical: align_position(
            &config.vertical,
            window_dimensions.height,
            screen_dimensions.height,
        ) as u32,
    };

    (window_dimensions, window_position)
}

fn get_screen_geometry() -> Dimensions {
    let geometry = gdk::Display::default()
        .expect("No default display")
        .primary_monitor()
        .expect("No default monitor")
        .geometry();

    Dimensions {
        height: geometry.height as u32,
        width: geometry.width as u32,
    }
}
