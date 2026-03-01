use super::Context;
use super::layer_navigation::run_layer_navigation_mode;
use crate::config::ApplicationConfig;
use crate::port::view::{ErrorViewModel, ViewAction, ViewModel};

/// Waits idly for the program activation signal and then moves to layer navigation.
pub fn run_inactive_mode(ctx: &Context) {
    let activation_keys: Vec<_> = ctx
        .keys_activate
        .iter()
        .chain(ctx.keys_app_activate.iter())
        .cloned()
        .collect();

    let press = ctx.input.capture_one(&activation_keys);
    if ctx.keys_activate.contains(&press) {
        run_layer_navigation_mode(ctx);
    } else {
        run_app_shortcut_mode(ctx, ctx.app_specific_layers);
    }

    ctx.view.render(ViewModel::None);
}

fn run_app_shortcut_mode(ctx: &Context, app_layers: &[ApplicationConfig]) {
    let window_name = ctx
        .window_manager
        .get_active_window_name()
        .unwrap_or_default();

    let Some(app_config) = app_layers.iter().find(|a| a.pattern.is_match(&window_name)) else {
        show_app_not_configured_error(ctx, &window_name);
        return;
    };

    run_layer_navigation_mode(&Context {
        executor: ctx.executor,
        input: ctx.input,
        view: ctx.view,
        window_manager: ctx.window_manager,
        keys_activate: ctx.keys_app_activate,
        keys_app_activate: &[],
        keys_back: ctx.keys_back,
        keys_deactivate: ctx.keys_deactivate,
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
    let error_details = format!(
        "No matching configuration found for \"{}\" application",
        window_label
    );
    let mut actions = vec![];
    for key in ctx.keys_back {
        actions.push((key, ViewAction::Unbranch));
    }
    for key in ctx.keys_deactivate {
        actions.push((key, ViewAction::Deactivate));
    }
    ctx.view.render(ViewModel::Error(ErrorViewModel {
        error_type: "Application unconfigured",
        error_details: &error_details,
        actions: &actions,
    }));
    loop {
        let press = ctx.input.capture_any();
        if ctx.keys_deactivate.contains(&press) || ctx.keys_back.contains(&press) {
            break;
        }
    }
}
