use super::viewmodel as VM;
use crate::gtk::config::Config;
use crate::gtk::viewmodel::ViewModel;
use gdk::prelude::*;
use gdk::{Display, Rectangle, WindowTypeHint};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Label, Orientation};

pub struct Gui {
    config: Config,
    ui: UiRoot,
    window: ApplicationWindow,
}

impl Gui {
    pub fn new(application: &Application, config: Config) -> Self {
        let window = ApplicationWindow::new(application);

        // Configuring the window properties.
        window.set_title("Ordinator");

        // Disabling the focusability of the window.
        window.set_accept_focus(false);
        window.set_can_focus(false);
        window.set_focus_on_click(false);
        window.set_focus_on_map(false);

        // Visual style.
        window.set_size_request(800, 400);
        window.set_modal(true);

        // Building of components
        let ui = UiRoot::new();
        window.add(&ui.build(&config));

        return Gui { config, window, ui };
    }

    pub fn update(&self, state: &VM::ViewModel) {
        if state.visible {
            self.window.show_all();
            self.ui.render(state);
        } else {
            self.window.hide();
        }
    }
}

fn get_display_geometry() -> Option<Rectangle> {
    Display::get_default()
        .and_then(|display| display.get_primary_monitor())
        .and_then(|monitor| Some(monitor.get_workarea()))
}

trait Component<Props> {
    fn new() -> Self;
    fn build(&self, config: &Config) -> gtk::Box;
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

    fn build(&self, config: &Config) -> Box {
        let mut component = Box::new(Orientation::Horizontal, 0);
        component.set_property_margin(config.padding as i32);
        component.add(&self.continuations.build(config));
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

    fn build(&self, config: &Config) -> Box {
        let component = Box::new(Orientation::Horizontal, config.padding as i32);
        for label in self.labels.iter() {
            component.add(&label.build(config));
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

    fn build(&self, config: &Config) -> gtk::Box {
        let component = Box::new(Orientation::Horizontal, (config.padding / 2) as i32);
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
