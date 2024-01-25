mod callback;
mod keybinding;
mod util;

use std::{sync::Arc, collections::HashMap, ffi::CString};

use xcb_util::{ewmh, keysyms};

use crate::{
    util as global_utils, // TODO: change this
    errors::Error,
    event::EventContext,
};

pub use crate::keyboard::{
    callback::FnOnKeypress,
    keybinding::{
        Keybinding,
        KeybindingBuilder,
    },
    util::KeyCombination,
};

pub struct Keyboard {
    conn: Arc<ewmh::Connection>,

    // TODO: There is probably a better way to hash the keypress action without a struct for this.
    actions: HashMap<KeyCombination, Box<dyn FnOnKeypress>>,
}

impl Keyboard {
    pub fn new(conn: Arc<ewmh::Connection>) -> Self {
        Self {
            conn,
            actions: HashMap::new(),
        }
    }

    pub fn trigger(&self, ctx: EventContext, combination: KeyCombination) -> Result<(), Error> {
        match self.actions.get(&combination) {
            Some(cb) => cb.call(ctx),
            None => Err(Error::Custom("hahaha".to_owned())),
        }
    }

    fn grab_key(
        &self,
        key_symbols: &keysyms::KeySymbols,
        modifier: u16,
        key: &str
    ) -> Result<u8, String> {
        let keysym = unsafe {
            let c_str = CString::new(key.to_owned()).map_err(|e| e.to_string())?;
            x11::xlib::XStringToKeysym(c_str.as_ptr()) as u32
        };

        let keycode = match key_symbols.get_keycode(keysym).next() {
            Some(keycode) => keycode,
            None => return Err(format!("Keycode for \"{}[{}]\" not found.", key, keysym).to_owned()),
        };

        xcb::grab_key(
            &self.conn,
            false,
            global_utils::get_screen(&self.conn).root(),
            modifier,
            keycode,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
        );

        Ok(keycode)
    }

    pub fn append_keybindings(&mut self, keybindings: &[Keybinding]) {
        let key_symbols = keysyms::KeySymbols::new(&self.conn);

        for kb in keybindings.iter() {
            let keycode = self.grab_key(&key_symbols, kb.modkeys, kb.key.as_str()).unwrap();

            let combination = KeyCombination {
                keycode,
                modifier: kb.modkeys,
            };

            self.actions.insert(combination, dyn_clone::clone_box(&*kb.callback));
        }

        self.conn.flush();
    }
}
