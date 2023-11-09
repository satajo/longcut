use crate::context::Context;

#[derive(Clone, Copy)]
pub enum Unit {
    /// Size in absolute pixels.
    Px(u32),

    /// Size relative to the line height of the current font.
    Em(f32),
}

impl Unit {
    pub fn as_pixel(&self, ctx: &Context) -> u32 {
        match self {
            Unit::Px(px) => *px,
            Unit::Em(em) => (em * ctx.font.size as f32) as u32,
        }
    }
}
