pub mod model;

use model::Model;

pub enum ControllerEvent {
    Begin,
    End,
}

pub trait Controller {
    fn read_event(&self) -> ControllerEvent;
}

pub trait View {
    fn render(&self, model: &Model);
}

pub fn main(controller: &impl Controller, view: &impl View) {
    let mut model = Model::new();
    loop {
        let input = controller.read_event();
        match input {
            ControllerEvent::Begin => {
                model.show();
            }
            ControllerEvent::End => {
                model.hide();
            }
        }
        view.render(&model);
    }
}
