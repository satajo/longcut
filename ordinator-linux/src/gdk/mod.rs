mod config;
mod gui;
mod renderer;
mod view_model;
mod window;

use crate::gdk::config::{Alignment, Color, Config, Dimensions, WindowConfig};
use crate::gdk::gui::Gui;
use crate::gdk::view_model::ViewModel;
use crate::gdk::window::Window;
use ordinator_core::port::view::{LayerViewData, View, ViewState};
use std::sync::mpsc::{channel, Sender};
use std::thread;

fn default_config() -> Config {
    Config {
        color_bg: Color::rgb(30, 30, 30),
        color_fg: Color::rgb(136, 255, 136),
        window: WindowConfig {
            size: Dimensions {
                vertical: 480,
                horizontal: 1280,
            },
            horizontal: Alignment::Center,
            vertical: Alignment::End,
        },
    }
}

pub struct GdkApplication {
    gdk_main_thread: Option<thread::JoinHandle<()>>,
    sender: Sender<ViewModel>,
}

impl GdkApplication {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<ViewModel>();
        let config = default_config();

        let gdk_main_thread = thread::spawn(move || {
            gdk::init();
            let window = Window::new(&config);
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
    fn render(&self, state: &ViewState) {
        self.sender
            .send(ViewModel::from(state))
            .expect("Failed to send ViewModel!")
    }
}
