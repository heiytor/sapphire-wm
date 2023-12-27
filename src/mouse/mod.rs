use std::sync::Arc;

use xcb_util::ewmh;

pub struct Mouse {
    conn: Arc<ewmh::Connection>,
}

impl Mouse {
    pub fn new(conn: Arc<ewmh::Connection>) -> Self {
        Mouse { conn }
    }
}

impl Mouse {
    /// Disables sloppy focus. Sloppy focus is a feature where the focused window follows the
    /// mouse pointer.
    pub fn disable_sloppy_focus(&self) {
        // We can disable it by simply calling "set_input_focus". The input focus needs to be
        // the root window to avoid keyboard blocks.
        xcb::set_input_focus(
            &self.conn,
            xcb::INPUT_FOCUS_PARENT as u8,
            xcb::INPUT_FOCUS_POINTER_ROOT,
            xcb::CURRENT_TIME,
        );
    }   
}
