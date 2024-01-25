use std::{ffi::CString, u16};

use xcb_util::keysyms;

use crate::{
    event::EventContext,
    errors::Error,
};

pub trait FnOnKeypress: dyn_clone::DynClone {
    fn call(&self, ctx: EventContext) -> Result<(), Error>;
}

impl<F> FnOnKeypress for F
where 
    F: Fn(EventContext) -> Result<(), Error> + Clone 
{
    fn call(&self, ctx: EventContext) -> Result<(), Error> {
        self(ctx)
    }
}

pub struct OnKeypress {
    callback: Box<dyn FnOnKeypress>,
    modifiers: Vec<u16>,
    key: String,
    keycode: u8,
}

#[derive(Hash, PartialEq, Eq)]
pub struct KeyCombination {
    pub keycode: u8,
    pub modifier: u16,
}

impl OnKeypress {
    pub fn new(
        modkey: &[u16],
        keys: &str,
        callback: Box<dyn FnOnKeypress>,
    ) -> Self {
        OnKeypress { 
            modifiers: modkey.to_vec(),
            key: keys.to_owned(),
            callback,
            keycode: 0,
        }
    }

    pub fn mask(&self) -> KeyCombination {
        KeyCombination {
            keycode: self.keycode,
            modifier: self.modifier(),
        }
    }

    #[inline]
    pub fn modifier(&self) -> u16 {
        self.modifiers.iter().fold(0, |acc, &m| acc | m)
    }

    /// TODO: grab compound keys... REFACTOR
    pub fn keycode(&mut self, sym: &keysyms::KeySymbols) -> Result<u8, String> {
        let keysym = unsafe {
            let c_str = CString::new(self.key.to_owned()).map_err(|e| e.to_string())?;
            x11::xlib::XStringToKeysym(c_str.as_ptr()) as u32
        };

        match sym.get_keycode(keysym).next() {
            Some(keycode) => {
                self.keycode = keycode;
                Ok(keycode)
            },
            None => Err(format!("Keycode for \"{}\" not found.", self.key).to_owned()),
        }
    }
}

impl Clone for OnKeypress {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            keycode: self.keycode.clone(),
            modifiers: self.modifiers.clone(),
            callback: dyn_clone::clone_box(&*self.callback),
        }
    }
}

impl OnKeypress {
    #[inline]
    pub fn call(&self, ctx: EventContext) -> Result<(), Error> {
        self.callback.call(ctx)
    }
}
