use x11rb::protocol::xproto::{Screen, Visualtype};

/// A `#[repr(C)]` struct matching the layout of `xcb_visualtype_t`, used to bridge between
/// x11rb's `Visualtype` and cairo's `XCBVisualType`.
#[repr(C)]
pub struct CXcbVisualtype {
    pub visual_id: u32,
    pub class: u8,
    pub bits_per_rgb_value: u8,
    pub colormap_entries: u16,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub pad0: [u8; 4],
}

impl CXcbVisualtype {
    pub fn from_x11rb(v: &Visualtype) -> Self {
        Self {
            visual_id: v.visual_id,
            class: u8::from(v.class),
            bits_per_rgb_value: v.bits_per_rgb_value,
            colormap_entries: v.colormap_entries,
            red_mask: v.red_mask,
            green_mask: v.green_mask,
            blue_mask: v.blue_mask,
            pad0: [0; 4],
        }
    }
}

/// Finds a 32-bit depth visual with an ARGB TrueColor configuration for transparency support.
pub fn find_argb_visual(screen: &Screen) -> Option<Visualtype> {
    for depth in &screen.allowed_depths {
        if depth.depth == 32 {
            for visual in &depth.visuals {
                if visual.class == x11rb::protocol::xproto::VisualClass::TRUE_COLOR
                    && visual.bits_per_rgb_value == 8
                {
                    return Some(*visual);
                }
            }
        }
    }
    None
}

/// Finds the visual matching the screen's root visual ID.
pub fn find_root_visual(screen: &Screen) -> Option<Visualtype> {
    for depth in &screen.allowed_depths {
        for visual in &depth.visuals {
            if visual.visual_id == screen.root_visual {
                return Some(*visual);
            }
        }
    }
    None
}
