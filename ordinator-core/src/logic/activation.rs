use crate::logic::layer_stack::LayerStackProgram;
use crate::logic::Context;
use crate::port::view::ViewState;

pub struct ActivationProgram;

impl ActivationProgram {
    pub fn run(ctx: &Context) {
        loop {
            ctx.input.capture_one(&ctx.keys_activate);
            LayerStackProgram::run(ctx);
            ctx.view.render(&ViewState::Hidden);
        }
    }
}
