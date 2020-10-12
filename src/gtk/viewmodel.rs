use crate::core::model::Model;

pub struct Continuation {
    shortcut: String,
    name: String,
}

pub struct ViewModel {
    sequence: Vec<Continuation>,
    continuations: Vec<Continuation>,
}

impl ViewModel {
    pub fn empty() -> ViewModel {
        return ViewModel {
            sequence: Vec::new(),
            continuations: Vec::new(),
        };
    }

    pub fn from_model(model: &Model) -> ViewModel {
        let mut continuations = Vec::new();
        if model.visible {
            continuations.push(Continuation {
                shortcut: "h".to_string(),
                name: "hide".to_string(),
            })
        } else {
            continuations.push(Continuation {
                shortcut: "s".to_string(),
                name: "show".to_string(),
            })
        }

        ViewModel {
            sequence: Vec::new(),
            continuations,
        }
    }
}
