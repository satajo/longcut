use crate::handle::GdkHandle;
use std::sync::mpsc::{channel, Sender};
use std::thread;

pub struct GdkService {
    gdk_main_thread: Option<thread::JoinHandle<()>>,
    sender: Sender<GdkOperation>,
}

pub type GdkOperation = Box<dyn FnOnce(&mut GdkHandle) + Send>;

impl GdkService {
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

        GdkService {
            gdk_main_thread: Some(gdk_main_thread),
            sender,
        }
    }
}

impl GdkService {
    pub fn run_in_gdk_thread(&self, operation: GdkOperation) {
        self.sender
            .send(operation)
            .expect("Failed to run operation in gdk main thread!")
    }
}

impl Drop for GdkService {
    fn drop(&mut self) {
        self.gdk_main_thread.take().unwrap().join().unwrap();
    }
}
