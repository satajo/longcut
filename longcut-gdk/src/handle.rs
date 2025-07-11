use crate::window::Window;
use gdk::prelude::MonitorExt;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

/// Provides access to Gdk library methods and objects.
pub struct GdkHandle {
    pub windows: GdkObjectStore<Window>,
}

impl GdkHandle {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        gdk::init();
        Self {
            windows: GdkObjectStore::new(),
        }
    }

    pub fn get_screen_dimensions(&self) -> (u32, u32) {
        let geometry = gdk::Display::default()
            .expect("No default display")
            .primary_monitor()
            .expect("No default monitor")
            .geometry();

        (geometry.width() as u32, geometry.height() as u32)
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct GdkObjectHandle(u32);

#[derive(Default)]
pub struct GdkObjectStore<T> {
    objects: BTreeMap<GdkObjectHandle, T>,
    next_id: u32,
}

impl<T> GdkObjectStore<T> {
    pub fn new() -> Self {
        Self {
            objects: BTreeMap::new(),
            next_id: 0,
        }
    }

    pub fn get_mut(&mut self, handle: &GdkObjectHandle) -> Option<&mut T> {
        self.objects.get_mut(handle)
    }

    pub fn insert(&mut self, item: T) -> (GdkObjectHandle, &mut T) {
        let handle = self.new_unique_handle();
        match self.objects.entry(handle) {
            Entry::Vacant(e) => (handle, &mut *e.insert(item)),
            Entry::Occupied(e) => (handle, &mut *e.into_mut()),
        }
    }

    pub fn remove(&mut self, handle: GdkObjectHandle) {
        self.objects.remove(&handle);
    }

    fn new_unique_handle(&mut self) -> GdkObjectHandle {
        let handle = GdkObjectHandle(self.next_id);
        self.next_id += 1;
        handle
    }
}
