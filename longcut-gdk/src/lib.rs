pub mod adapter;

mod component;
mod config;
mod gui;
mod renderer;
mod screen;
mod window;

use crate::config::Config;
use crate::gui::Gui;
use crate::window::Window;
use gui::GuiState;

use std::sync::mpsc::{channel, Sender};
use std::thread;

pub struct GdkModule {
    gdk_main_thread: Option<thread::JoinHandle<()>>,
    sender: Sender<GuiState>,
}

impl GdkModule {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<GuiState>();
        let config = Config::default();

        let gdk_main_thread = thread::spawn(move || {
            gdk::init();
            let window = Window::new(&config.window);
            let gui = Gui::new(&config, &window);
            loop {
                let data = receiver.recv().expect("Failed to recv view update channel");
                gui.update(data);
            }
        });

        GdkModule {
            gdk_main_thread: Some(gdk_main_thread),
            sender,
        }
    }
}

impl Drop for GdkModule {
    fn drop(&mut self) {
        self.gdk_main_thread.take().unwrap().join().unwrap();
    }
}
