mod config;
mod gui;
mod view_model;

use crate::gtk::config::Config;
use gio::prelude::*;
use gtk::Application;
use gui::Gui;
use ordinator_core::port::view::{View, ViewData};
use std::sync::mpsc;
use std::thread;
use view_model::ViewModel;

pub struct GtkApplication {
    view_updates: glib::Sender<ViewModel>,
}

impl GtkApplication {
    pub fn new() -> GtkApplication {
        let (view_sender_sender, view_sender_receiver) = mpsc::channel::<glib::Sender<ViewModel>>();

        // Gtk app is launched in its own thread.
        thread::spawn(move || {
            let application = Application::new(None, Default::default());
            application.connect_activate(move |application| {
                let gui = Gui::new(&application, Config::new());
                let (view_sender, view_receiver) =
                    glib::MainContext::channel::<ViewModel>(glib::PRIORITY_DEFAULT);
                view_sender_sender.send(view_sender).unwrap();
                view_receiver.attach(None, move |msg| {
                    gui.update(&msg);
                    Continue(true)
                });
            });

            println!("Running gtk loop!");
            application.run()
        });

        // Gtk application start is awaited by waiting for the view sender being sent.
        let view_sender = view_sender_receiver.recv().unwrap();

        // References for outward communication are returned.
        GtkApplication {
            view_updates: view_sender,
        }
    }
}

impl View for GtkApplication {
    fn render(&self, data: &ViewData) {
        self.view_updates.send(ViewModel::new(data)).unwrap();
    }
}
