use crate::gdk::component::{Component, Context};
use crate::gdk::config::Dimensions;

pub struct Rectangle<C: Component> {
    child: C,
    padding_bottom: u32,
    padding_left: u32,
    padding_right: u32,
    padding_top: u32,
}

impl<C: Component> Rectangle<C> {
    pub fn new(child: C) -> Self {
        Self {
            child,
            padding_bottom: 0,
            padding_left: 0,
            padding_right: 0,
            padding_top: 0,
        }
    }

    pub fn pad(self, amount: u32) -> Self {
        self.pad_horizontal(amount).pad_vertical(amount)
    }

    pub fn pad_horizontal(self, amount: u32) -> Self {
        self.pad_left(amount).pad_right(amount)
    }

    pub fn pad_vertical(self, amount: u32) -> Self {
        self.pad_bottom(amount).pad_right(amount)
    }

    pub fn pad_bottom(mut self, padding: u32) -> Self {
        self.padding_bottom = padding;
        self
    }

    pub fn pad_left(mut self, padding: u32) -> Self {
        self.padding_left = padding;
        self
    }

    pub fn pad_right(mut self, padding: u32) -> Self {
        self.padding_right = padding;
        self
    }

    pub fn pad_top(mut self, padding: u32) -> Self {
        self.padding_top = padding;
        self
    }
}

impl<C: Component> Component for Rectangle<C> {
    fn render(&self, context: &Context) {
        let offset = Dimensions::new(self.padding_left, self.padding_right);
        let padded_context = context.offset(offset);
        self.child.render(&padded_context);
    }

    fn measure(&self, context: &Context) -> Dimensions {
        let padding = Dimensions {
            horizontal: self.padding_left + self.padding_right,
            vertical: self.padding_bottom + self.padding_top,
        };

        padding + self.child.measure(context)
    }
}
