use std::{collections::HashMap, sync::Arc};

use xcb_util::{ewmh, keysyms};

use crate::{action::{on_startup::OnStartupAction, on_keypress::OnKeypressAction}, util};

pub struct Actions {
    conn: Arc<ewmh::Connection>,

    pub at_startup: Vec<OnStartupAction>,
    pub at_keypress: HashMap<u8, OnKeypressAction>
}

impl Actions {
    pub fn new(conn: Arc<ewmh::Connection>) -> Self {
        Actions {
            conn,
            at_startup: Vec::new(),
            at_keypress: HashMap::new(),
        }
    }
}

impl Actions {
    #[inline]
    pub fn new_on_startup(&mut self, action: OnStartupAction) {
        self.at_startup.push(action);
    }

    #[inline]
    pub fn new_on_keypress(&mut self, action: OnKeypressAction) {
        let key_symbols = keysyms::KeySymbols::new(&self.conn);
        match key_symbols.get_keycode(util::to_keysym(action.ch)).next() {
            Some(keycode) => self.at_keypress.insert(keycode, action),
            _ => panic!("Failed to find keycode for char: {}", action.ch),
        };
    }
}
