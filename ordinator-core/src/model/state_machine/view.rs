//-----------------------------------------------------------------------------
// View model transformations
//-----------------------------------------------------------------------------

use crate::model::state_machine::state::{Branch, Inactive, Root, State};
use crate::port::view::ViewData;

pub trait Viewable {
    fn to_view_data(&self) -> ViewData;
}

impl Viewable for State<Inactive> {
    fn to_view_data(&self) -> ViewData {
        ViewData {
            visible: false,
            actions: vec![],
        }
    }
}

impl Viewable for State<Branch> {
    fn to_view_data(&self) -> ViewData {
        let mut actions = vec![];
        for (press, action) in &self.state.branches.last().unwrap().actions {
            actions.push((press.clone(), action.clone()))
        }
        ViewData {
            visible: true,
            actions,
        }
    }
}

impl Viewable for State<Root> {
    fn to_view_data(&self) -> ViewData {
        let mut actions = vec![];
        for (press, action) in &self.root.actions {
            actions.push((press.clone(), action.clone()))
        }
        ViewData {
            visible: true,
            actions,
        }
    }
}
