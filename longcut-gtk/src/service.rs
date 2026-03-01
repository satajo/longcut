use crate::handle::GtkHandle;
use gdk4::glib;
use std::cell::RefCell;
use std::sync::mpsc;
use std::thread;

thread_local! {
    static GTK_HANDLE: RefCell<Option<GtkHandle>> = const { RefCell::new(None) };
}

pub struct GtkService {
    gtk_main_thread: Option<thread::JoinHandle<()>>,
    main_context: glib::MainContext,
}

pub type GtkOperation = Box<dyn FnOnce(&mut GtkHandle) + Send>;

impl GtkService {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (init_sender, init_receiver) = mpsc::channel::<()>();

        let gtk_main_thread = thread::spawn(move || {
            gtk4::init().expect("Failed to initialize GTK4");

            let main_loop = glib::MainLoop::new(None, false);

            GTK_HANDLE.with(|h| *h.borrow_mut() = Some(GtkHandle::new()));

            init_sender
                .send(())
                .expect("Failed to signal GTK initialization complete");

            main_loop.run();
        });

        init_receiver
            .recv()
            .expect("Failed to wait for GTK initialization");

        GtkService {
            gtk_main_thread: Some(gtk_main_thread),
            main_context: glib::MainContext::default(),
        }
    }
}

impl GtkService {
    pub fn run_in_gtk_thread(&self, operation: GtkOperation) {
        self.main_context.invoke(move || {
            GTK_HANDLE.with(|h| {
                let mut guard = h.borrow_mut();
                let handle = guard.as_mut().expect("GtkHandle not initialized");
                operation(handle);
            });
        });
    }
}

impl Drop for GtkService {
    fn drop(&mut self) {
        self.gtk_main_thread.take().unwrap().join().unwrap();
    }
}
