use super::Context;
use super::layer_navigation::run_layer_navigation_mode;
use crate::port::view::ViewModel;

/// Waits idly for the program activation signal and then moves to layer navigation.
pub fn run_inactive_mode(ctx: &Context) {
    ctx.input.capture_one(ctx.keys_activate);
    run_layer_navigation_mode(ctx);
    ctx.view.render(ViewModel::None);
}
