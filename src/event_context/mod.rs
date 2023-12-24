use std::sync::Arc;

use xcb_util::ewmh;

pub struct EventContext {
    pub conn: Arc<ewmh::Connection>,
    pub screen: i32,
}

impl EventContext {
    pub fn new(conn: Arc<ewmh::Connection>, screen: i32) -> Self {
        EventContext { conn, screen }
    }
}

impl EventContext {
    pub fn get_active_window(&self) -> Result<u32, String> {
        let active_window = ewmh::get_active_window(&self.conn, self.screen)
            .get_reply()
            .map_err(|e| format!("Failed to get active window. Error: {:?}", e))?;

        Ok(active_window)
    }
}
