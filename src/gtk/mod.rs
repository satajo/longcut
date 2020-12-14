mod gui;
mod viewmodel;

use crate::core::model::Model;
use crate::core::View;
use gio::prelude::*;
use gtk::Application;
use gui::Gui;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use viewmodel::ViewModel;

struct ViewEvent {}

pub struct GtkApplication {
    view_updates: glib::Sender<ViewModel>,
    command_receiver: Receiver<ViewEvent>,
}

impl GtkApplication {
    pub fn new() -> GtkApplication {
        let (_command_sender, command_receiver) = mpsc::channel::<ViewEvent>();
        let (view_sender_sender, view_sender_receiver) = mpsc::channel::<glib::Sender<ViewModel>>();

        // Gtk app is launched in its own thread.
        thread::spawn(move || {
            let application = Application::new(None, Default::default())
                .expect("Failed to initialize application");
            application.connect_activate(move |application| {
                let gui = Gui::new(&application);
                let (view_sender, view_receiver) =
                    glib::MainContext::channel::<ViewModel>(glib::PRIORITY_DEFAULT);
                view_sender_sender.send(view_sender).unwrap();
                view_receiver.attach(None, move |msg| {
                    gui.update(&msg);
                    return Continue(true);
                });
            });

            println!("Running gtk loop!");
            application.run(&[])
        });

        let view_sender = view_sender_receiver.recv().unwrap();

        // References for outward communication are returned.
        return GtkApplication {
            view_updates: view_sender,
            command_receiver,
        };
    }
}

impl View for GtkApplication {
    fn render(&self, model: &Model) {
        self.view_updates
            .send(ViewModel::from_model(model))
            .unwrap();
    }
}

// pub struct KeyPressEvent {}
// fn connect_keypress_handler<T: App>(window: &ApplicationWindow, app: &'static T) {
//     window.connect_key_press_event(|_, event| {
//         let keycode = event.get_hardware_keycode();
//         app.handle_keypress(keycode);
//         return Inhibit(false);
//     });
// }
