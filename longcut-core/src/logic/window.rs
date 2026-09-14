use super::Context;
use super::layer_navigation::run_layer_navigation_mode;
use crate::port::view::{ErrorViewModel, ViewAction, ViewModel};

/// Navigates the layer configured for the currently active window.
///
/// The active window is resolved through the window manager and matched against the configured
/// application patterns. When no pattern matches, the user is shown the "application unconfigured"
/// error instead.
pub fn run_window_mode(ctx: &Context) {
    let window_name = ctx
        .window_manager
        .get_active_window_name()
        .unwrap_or_default();

    let Some(app_config) = ctx
        .app_specific_layers
        .iter()
        .find(|a| a.pattern.is_match(&window_name))
    else {
        show_app_not_configured_error(ctx, &window_name);
        return;
    };

    run_layer_navigation_mode(&Context {
        executor: ctx.executor,
        keyboard: ctx.keyboard,
        view: ctx.view,
        window_manager: ctx.window_manager,
        keys_back: ctx.keys_back,
        keys_exit: ctx.keys_exit,
        keys_retry: ctx.keys_retry,
        root_layer: &app_config.root_layer,
        app_specific_layers: &[],
    });
}

fn show_app_not_configured_error(ctx: &Context, window_name: &str) {
    let window_label = if window_name.is_empty() {
        "unknown"
    } else {
        window_name
    };
    let error_details =
        format!("No matching configuration found for \"{window_label}\" application");
    let mut actions = vec![];
    for key in ctx.keys_back {
        actions.push((key, ViewAction::Unbranch));
    }
    for key in ctx.keys_exit {
        actions.push((key, ViewAction::Exit));
    }
    ctx.view.render(ViewModel::Error(ErrorViewModel {
        error_type: "Application unconfigured",
        error_details: &error_details,
        actions: &actions,
    }));
    loop {
        let press = ctx.keyboard.capture_any();
        if ctx.keys_exit.contains(&press) || ctx.keys_back.contains(&press) {
            break;
        }
    }
}
