use super::Context;
use super::layer_navigation::run_layer_navigation_mode;
use crate::port::view::{ErrorViewModel, ViewAction, ViewModel};

/// Navigates the layer configured for the currently active window.
///
/// The active window is resolved through the window manager and matched against the configured
/// application patterns. When no pattern matches, the user is shown the "application unconfigured"
/// error instead.
pub(crate) fn run_window_mode(ctx: &Context) {
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
        clock: ctx.clock,
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

#[cfg(test)]
mod tests {
    use super::run_window_mode;
    use crate::model::key::{Key, Symbol};
    use crate::model::layer::Layer;
    use crate::port::fakes::Event::{ActiveWindow, KeyPressed};
    use crate::port::fakes::{Effect, Scenario, Screen};

    fn back_key() -> Key {
        Key::new(Symbol::BACKSPACE)
    }

    fn exit_key() -> Key {
        Key::new(Symbol::ESCAPE)
    }

    fn shows_layers(layer_stack: &[&str]) -> Effect {
        Effect::Rendered(Screen::LayerNavigation {
            layer_stack: layer_stack.iter().map(ToString::to_string).collect(),
        })
    }

    fn shows_application_unconfigured() -> Effect {
        Effect::Rendered(Screen::Error {
            error_type: "Application unconfigured".to_string(),
            retry_offered: false,
        })
    }

    #[test]
    fn active_window_matching_a_pattern_navigates_that_applications_layer() {
        Scenario::new()
            .app_layer("^Mozilla", Layer::new("Thunderbird".to_string()))
            .app_layer("Firefox$", Layer::new("Firefox".to_string()))
            .given(ActiveWindow("Mozilla Firefox".to_string()))
            .run(run_window_mode)
            .expect(shows_layers(&["Thunderbird"]))
            .then(KeyPressed(exit_key()))
            .returns(());
    }

    #[test]
    fn active_window_matching_no_pattern_shows_the_unconfigured_error_until_dismissed() {
        Scenario::new()
            .app_layer("Firefox$", Layer::new("Firefox".to_string()))
            .given(ActiveWindow("Alacritty".to_string()))
            .run(run_window_mode)
            .expect(shows_application_unconfigured())
            .then(KeyPressed(Key::new(Symbol::from_char('x').unwrap())))
            .then(KeyPressed(back_key()))
            .returns(());
    }

    #[test]
    fn no_active_window_shows_the_unconfigured_error() {
        Scenario::new()
            .app_layer("Firefox$", Layer::new("Firefox".to_string()))
            .run(run_window_mode)
            .expect(shows_application_unconfigured())
            .then(KeyPressed(back_key()))
            .returns(());
    }
}
