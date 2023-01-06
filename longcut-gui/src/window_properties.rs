use longcut_graphics_lib::model::alignment::Alignment2d;
use longcut_graphics_lib::model::dimensions::Dimensions;

#[derive(Clone, Debug)]
pub struct WindowProperties {
    pub size: Dimensions,
    pub alignment: Alignment2d,
}
