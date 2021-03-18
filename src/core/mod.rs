pub mod model;
pub mod port;

use crate::core::model::key::Key;
use crate::core::port::controller::Controller;
use crate::core::port::view::View;
use model::Model;

pub fn main(mut controller: impl Controller, mut view: impl View) {
    let begin_key = Key::from_keycode(115);

    let mut model = Model::new();
    loop {
        if !model.visible {
            controller.capture_one_of(&vec![begin_key.clone()]);
            model.show();
        } else {
            let press = controller.capture_all();
            println!("Pressed {:?}", press);
            model.hide();
        }

        view.render(&model);
    }
}
