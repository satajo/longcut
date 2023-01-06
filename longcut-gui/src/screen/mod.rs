use crate::screen::error::ErrorScreen;
use crate::screen::layer_navigation::LayerNavigationScreen;
use crate::screen::parameter_input::ParameterInputScreen;

pub mod error;
pub mod layer_navigation;
pub mod parameter_input;

pub enum Screen {
    LayerNavigation(LayerNavigationScreen),
    ParameterInput(ParameterInputScreen),
    Error(ErrorScreen),
}
