use crate::context::Context;

#[derive(Clone, Copy, Debug)]
pub enum Unit {
    /// Size in absolute pixels.
    Px(u32),

    /// Size relative to the line height of the current font.
    Em(f32),
}

impl Unit {
    #[must_use]
    pub fn as_pixel(self, ctx: &Context) -> u32 {
        match self {
            Unit::Px(px) => px,
            #[expect(
                clippy::as_conversions,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "em values are written as literals between 0.5 and 6 next to the components that use them and the font size is a u8, so the product is finite, non-negative and far below u32::MAX"
            )]
            Unit::Em(em) => (em * f32::from(ctx.font.size)).round() as u32,
        }
    }
}
