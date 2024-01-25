use std::sync::{Arc, Mutex};

use xcb_util::ewmh;

use crate::screen::Screen;

#[derive(Clone)]
pub struct EventContext {
    pub conn: Arc<ewmh::Connection>,
    pub screen: Arc<Mutex<Screen>>,
}

impl EventContext {
    pub fn new(conn: Arc<ewmh::Connection>, screen: Arc<Mutex<Screen>>) -> Self {
        Self {
            conn,
            screen,
        }
    }
}
