pub struct Model {
    pub visible: bool,
}

impl Model {
    pub fn new() -> Model {
        return Model { visible: false };
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }
}
