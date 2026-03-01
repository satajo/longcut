use crate::window::Window;
use gdk4::prelude::*;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

/// Provides access to GTK library methods and objects.
pub struct GtkHandle {
    pub windows: GtkObjectStore<Window>,
}

impl GtkHandle {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            windows: GtkObjectStore::new(),
        }
    }

    pub fn get_screen_dimensions(&self) -> (u32, u32) {
        let display = gdk4::Display::default().expect("No default display");
        let monitors = display.monitors();

        // Try to find the primary monitor (the one at origin 0,0) since GTK4 removed
        // the explicit primary_monitor() API. Fall back to the first monitor.
        let monitor = (0..monitors.n_items())
            .filter_map(|i| monitors.item(i))
            .filter_map(|obj| obj.downcast::<gdk4::Monitor>().ok())
            .find(|m| {
                let geo = m.geometry();
                geo.x() == 0 && geo.y() == 0
            })
            .or_else(|| {
                monitors
                    .item(0)
                    .and_then(|obj| obj.downcast::<gdk4::Monitor>().ok())
            })
            .expect("No monitors found");

        let geometry = monitor.geometry();
        (geometry.width() as u32, geometry.height() as u32)
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct GtkObjectHandle(u32);

#[derive(Default)]
pub struct GtkObjectStore<T> {
    objects: BTreeMap<GtkObjectHandle, T>,
    next_id: u32,
}

impl<T> GtkObjectStore<T> {
    pub fn new() -> Self {
        Self {
            objects: BTreeMap::new(),
            next_id: 0,
        }
    }

    pub fn get_mut(&mut self, handle: &GtkObjectHandle) -> Option<&mut T> {
        self.objects.get_mut(handle)
    }

    pub fn insert(&mut self, item: T) -> (GtkObjectHandle, &mut T) {
        let handle = self.new_unique_handle();
        match self.objects.entry(handle) {
            Entry::Vacant(e) => (handle, &mut *e.insert(item)),
            Entry::Occupied(e) => (handle, &mut *e.into_mut()),
        }
    }

    pub fn remove(&mut self, handle: GtkObjectHandle) {
        self.objects.remove(&handle);
    }

    fn new_unique_handle(&mut self) -> GtkObjectHandle {
        let handle = GtkObjectHandle(self.next_id);
        self.next_id += 1;
        handle
    }
}
