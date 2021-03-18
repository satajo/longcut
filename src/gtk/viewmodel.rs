use crate::core::model::Model;

pub struct Settings {
    pub padding: u16,
}

pub struct Continuation {
    pub shortcut: String,
    pub name: String,
}

pub struct ViewModel {
    pub visible: bool,
    pub sequence: Vec<Continuation>,
    pub continuations: Vec<Continuation>,
    pub settings: Settings,
}

impl ViewModel {
    pub fn empty() -> Self {
        return ViewModel {
            visible: false,
            sequence: Vec::new(),
            continuations: Vec::new(),
            settings: Settings { padding: 8 },
        };
    }

    pub fn from_model(model: &Model) -> Self {
        let mut vm = Self::empty();

        vm.visible = model.visible;
        vm.continuations.push(Continuation {
            shortcut: "h".to_string(),
            name: "hide".to_string(),
        });
        vm.continuations.push(Continuation {
            shortcut: "s".to_string(),
            name: "show".to_string(),
        });
        return vm;
    }
}
