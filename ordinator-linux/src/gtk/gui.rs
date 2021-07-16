use super::view_model as VM;
use crate::gtk::config::Config;
use crate::gtk::view_model::ViewModel;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Label, Orientation};

pub struct Gui {
    ui: UiRoot,
    window: ApplicationWindow,
}

impl Gui {
    pub fn new(application: &Application, config: Config) -> Self {
        let window = ApplicationWindow::new(application);
        window.set_title("Ordinator");

        // Interaction settings.
        window.set_accept_focus(false);
        window.set_can_focus(false);
        window.set_focus_on_click(false);
        window.set_focus_on_map(false);

        // Visual style.
        window.set_decorated(false);
        window.set_modal(true);

        // Building of components
        let ui = UiRoot::new();
        window.add(&ui.build(&config));

        Gui { ui, window }
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

trait Component<Props> {
    fn new() -> Self;
    fn build(&self, config: &Config) -> gtk::Box;
    fn render(&self, props: &Props);
}

struct UiRoot {
    continuations: Continuations,
    layer_name_row: LayerNameRow,
}

impl Component<VM::ViewModel> for UiRoot {
    fn new() -> Self {
        UiRoot {
            continuations: Continuations::new(),
            layer_name_row: LayerNameRow::new(),
        }
    }

    fn build(&self, config: &Config) -> Box {
        let component = Box::new(Orientation::Vertical, config.padding as i32);
        component.set_margin(config.padding as i32);
        component.add(&self.layer_name_row.build(config));
        component.add(&self.continuations.build(config));
        component
    }

    fn render(&self, props: &ViewModel) {
        self.layer_name_row.render(props);
        self.continuations.render(&props.continuations);
    }
}

struct LayerNameRow {
    name: gtk::Label,
}

impl Component<VM::ViewModel> for LayerNameRow {
    fn new() -> Self {
        Self {
            name: Label::new(None),
        }
    }

    fn build(&self, _config: &Config) -> Box {
        let component = Box::new(Orientation::Horizontal, 0);
        component.add(&self.name);
        component
    }

    fn render(&self, props: &ViewModel) {
        let layers = props.layer_stack.join(" > ");
        let text = format!("Layer {}", layers);
        self.name.set_label(&text);
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

    fn render(&self, props: &Vec<VM::Continuation>) {
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

    fn render(&self, props: &Option<&VM::Continuation>) {
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
