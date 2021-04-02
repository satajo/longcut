use super::viewmodel as VM;
use crate::gtk::viewmodel::ViewModel;
use gdk::{Display, Rectangle};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Label, Orientation};

pub struct Gui {
    window: ApplicationWindow,
    ui: UiRoot,
}

impl Gui {
    pub fn new(application: &Application) -> Self {
        let window = ApplicationWindow::new(application);
        let _geometry = get_display_geometry().expect("Unable to read display geometry!");

        // Building of components
        let ui = UiRoot::new();
        window.add(&ui.build());

        // Displaying the window
        window.set_size_request(800, 600);
        window.set_decorated(false);
        window.set_keep_above(true);
        window.set_modal(true);
        //window.show_all();
        return Gui { window, ui };
    }

    pub fn update(&self, state: &VM::ViewModel) {
        if state.visible {
            self.window.show_all();
        } else {
            self.window.hide();
            return;
        }

        self.ui.render(state);
    }
}

fn get_display_geometry() -> Option<Rectangle> {
    Display::get_default()
        .and_then(|display| display.get_primary_monitor())
        .and_then(|monitor| Some(monitor.get_workarea()))
}

trait Component<Props> {
    fn new() -> Self;
    fn build(&self) -> gtk::Box;
    fn render(&self, props: &Props) -> ();
}

struct UiRoot {
    continuations: Continuations,
}

impl Component<VM::ViewModel> for UiRoot {
    fn new() -> Self {
        UiRoot {
            continuations: Continuations::new(),
        }
    }

    fn build(&self) -> Box {
        let component = Box::new(Orientation::Horizontal, 0);
        component.add(&self.continuations.build());
        component
    }

    fn render(&self, props: &ViewModel) -> () {
        self.continuations.render(&props.continuations);
    }
}

struct Continuations {
    labels: Vec<Continuation>,
}

impl Component<Vec<VM::Continuation>> for Continuations {
    fn new() -> Self {
        let mut labels = Vec::new();
        for _ in 0..10 {
            labels.push(Continuation::new());
        }
        Continuations { labels }
    }

    fn build(&self) -> Box {
        let component = Box::new(Orientation::Horizontal, 16);
        for label in self.labels.iter() {
            component.add(&label.build());
        }
        component
    }

    fn render(&self, props: &Vec<VM::Continuation>) -> () {
        for (index, label) in self.labels.iter().enumerate() {
            label.render(&props.get(index));
        }
    }
}

struct Continuation {
    name: gtk::Label,
    shortcut: gtk::Label,
}

impl Component<Option<&VM::Continuation>> for Continuation {
    fn new() -> Self {
        Continuation {
            name: Label::new(None),
            shortcut: Label::new(None),
        }
    }

    fn build(&self) -> gtk::Box {
        let component = Box::new(Orientation::Horizontal, 8);
        component.add(&self.shortcut);
        component.add(&self.name);
        component
    }

    fn render(&self, props: &Option<&VM::Continuation>) -> () {
        match props {
            None => {
                self.name.set_label("");
                self.shortcut.set_label("");
            }
            Some(data) => {
                self.name.set_label(data.name.as_str());
                self.shortcut.set_label(data.shortcut.as_str());
            }
        }
    }
}
