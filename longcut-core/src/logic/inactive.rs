use super::Context;
use super::layer_navigation::run_layer_navigation_mode;
use crate::config::ApplicationShortcutsConfig;
use crate::port::view::ViewModel;

/// Waits idly for the program activation signal and then moves to layer navigation.
pub fn run_inactive_mode(ctx: &Context) {
    let activation_keys: Vec<_> = ctx
        .keys_activate
        .iter()
        .chain(ctx.keys_app_activate.iter())
        .cloned()
        .collect();

    let pressed = ctx.input.capture_one(&activation_keys);
    if ctx.keys_app_activate.contains(&pressed) {
        if let Some(app_cfg) = ctx.application_shortcuts {
            run_app_shortcut_mode(ctx, app_cfg);
        }
    } else {
        run_layer_navigation_mode(ctx);
    }

    ctx.view.render(ViewModel::None);
}

fn run_app_shortcut_mode(ctx: &Context, app_cfg: &ApplicationShortcutsConfig) {
    let window_name = ctx
        .window_manager
        .get_active_window_name()
        .unwrap_or_default();

    if let Some(app_config) = app_cfg
        .applications
        .iter()
        .find(|a| a.pattern.is_match(&window_name))
    {
        let sub_ctx = Context {
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
            application_shortcuts: None,
        };
        run_layer_navigation_mode(&sub_ctx);
    }
    // If no application matches, silently return.
}
