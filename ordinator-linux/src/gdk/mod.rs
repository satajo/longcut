mod component;
mod config;
mod gui;
mod renderer;
mod screen;
mod window;

use crate::gdk::config::Config;
use crate::gdk::gui::Gui;
use crate::gdk::window::Window;
use gui::GuiState;
use ordinator_core::port::view::{View, ViewModel};
use std::sync::mpsc::{channel, Sender};
use std::thread;

pub struct GdkApplication {
    gdk_main_thread: Option<thread::JoinHandle<()>>,
    sender: Sender<GuiState>,
}

impl GdkApplication {
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

        GdkApplication {
            gdk_main_thread: Some(gdk_main_thread),
            sender,
        }
    }
}

impl Drop for GdkApplication {
    fn drop(&mut self) {
        self.gdk_main_thread.take().unwrap().join().unwrap();
    }
}

impl View for GdkApplication {
    fn render(&self, state: ViewModel) {
        self.sender
            .send(GuiState::from(state))
            .expect("Failed to send ViewModel!")
    }
}
