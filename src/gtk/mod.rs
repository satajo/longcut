mod viewmodel;

use crate::core::model::Model;
use crate::core::{Controller, ControllerEvent, View};

use gdk::{Display, Rectangle};
use gio::prelude::*;
use gtk::{Application, ApplicationWindow, GtkApplicationExt, GtkWindowExt, Inhibit, WidgetExt};
use std::cell::RefCell;
use std::thread;
use std::time::Duration;
use viewmodel::ViewModel;

pub struct GtkApplication {
    contents: RefCell<ViewModel>,
}

impl GtkApplication {
    pub fn new() -> GtkApplication {
        let view = GtkApplication {
            contents: RefCell::new(ViewModel::empty()),
        };

        // Gtk app is launched in its own thread.
        launch_application_thread(&view);

        return view;
    }
}

impl View for GtkApplication {
    fn update(&self, model: &Model) {
        self.contents.replace(ViewModel::from_model(model));
    }
}

fn launch_application_thread(view: &GtkApplication) {
    thread::spawn(|| {
        let application = prepare_application();
        application.connect_activate(init_gui);
        application.run(&[])
    });
}

fn prepare_application() -> Application {
    Application::new(Some("ordinator.gui"), Default::default())
        .expect("Failed to initialize application")
}

fn init_gui(application: &Application) {
    let window = ApplicationWindow::new(application);
    // connect_keypress_handler(&window, app);
    show_gui(window);
}

// pub struct KeyPressEvent {}
// fn connect_keypress_handler<T: App>(window: &ApplicationWindow, app: &'static T) {
//     window.connect_key_press_event(|_, event| {
//         let keycode = event.get_hardware_keycode();
//         app.handle_keypress(keycode);
//         return Inhibit(false);
//     });
// }

fn show_gui(window: ApplicationWindow) {
    let geometry = get_display_geometry().expect("Unable to read display geometry!");
    window.set_size_request(geometry.width, 200);
    window.set_decorated(false);
    window.set_keep_above(true);
    window.show_all();
}

fn get_display_geometry() -> Option<Rectangle> {
    Display::get_default()
        .and_then(|display| display.get_primary_monitor())
        .and_then(|monitor| Some(monitor.get_workarea()))
}
