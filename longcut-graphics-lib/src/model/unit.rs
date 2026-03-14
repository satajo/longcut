use crate::context::Context;

#[derive(Clone, Copy)]
pub enum Unit {
    /// Size in absolute pixels.
    Px(u32),

    /// Size relative to the line height of the current font.
    Em(f32),
}

impl Unit {
    #[must_use]
    pub fn as_pixel(&self, ctx: &Context) -> u32 {
        match self {
            Unit::Px(px) => *px,
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "em values multiplied by font size always produce small positive results"
            )]
            Unit::Em(em) => (em * f32::from(ctx.font.size)) as u32,
        }
    }
}
