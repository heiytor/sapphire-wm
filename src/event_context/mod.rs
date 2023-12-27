use std::sync::{Arc, Mutex};

use xcb_util::ewmh;

use crate::client::{Clients, Dir};

pub struct EventContext {
    pub conn: Arc<ewmh::Connection>,
    pub screen: i32,
    pub clients: Arc<Mutex<Clients>>,
}

impl EventContext {
    pub fn new(conn: Arc<ewmh::Connection>, screen: i32, clients: Arc<Mutex<Clients>>) -> Self {
        EventContext { conn, screen, clients }
    }
}

impl EventContext {
    pub fn active_window(&self) -> Result<u32, String> {
        let active_window = ewmh::get_active_window(&self.conn, self.screen)
            .get_reply()
            .map_err(|_| format!("Failed to get active window. Error"))?;

        Ok(active_window)
    }

}
