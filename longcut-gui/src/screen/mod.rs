use crate::screen::error::ErrorScreen;
use crate::screen::layer_navigation::LayerNavigationScreen;
use crate::screen::parameter_input::ParameterInputScreen;

pub(crate) mod error;
pub(crate) mod layer_navigation;
pub(crate) mod parameter_input;

#[derive(Debug)]
pub enum Screen {
    LayerNavigation(LayerNavigationScreen),
    ParameterInput(ParameterInputScreen),
    Error(ErrorScreen),
}
