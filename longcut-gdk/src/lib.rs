use std::sync::mpsc::{channel, Sender};
use std::thread;

use crate::handle::GdkHandle;

pub mod adapter;
mod handle;
mod window;

pub type GdkOperation = Box<dyn FnOnce(&mut GdkHandle) + Send>;

pub struct GdkModule {
    gdk_main_thread: Option<thread::JoinHandle<()>>,
    sender: Sender<GdkOperation>,
}

impl GdkModule {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (sender, receiver) = channel::<GdkOperation>();

        let gdk_main_thread = thread::spawn(move || {
            let mut handle = GdkHandle::new();
            loop {
                let operation = receiver.recv().expect("Failed to recv view update channel");
                operation(&mut handle);
            }
        });

        GdkModule {
            gdk_main_thread: Some(gdk_main_thread),
            sender,
        }
    }
}

impl GdkModule {
    pub fn run_in_gdk_thread(&self, operation: GdkOperation) {
        self.sender
            .send(operation)
            .expect("Failed to run operation in gdk main thread!")
    }
}

impl Drop for GdkModule {
    fn drop(&mut self) {
        self.gdk_main_thread.take().unwrap().join().unwrap();
    }
}
