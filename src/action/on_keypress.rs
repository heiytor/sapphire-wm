use std::ffi::CString;

use xcb_util::keysyms;

use crate::event_context::EventContext;

pub trait FnOnKeypress: dyn_clone::DynClone {
    fn call(&self, ctx: EventContext) -> Result<(), String>;
}

impl<F> FnOnKeypress for F
where 
    F: Fn(EventContext) -> Result<(), String> + Clone 
{
    fn call(&self, ctx: EventContext) -> Result<(), String> {
        self(ctx)
    }
}

pub struct OnKeypress {
    callback: Box<dyn FnOnKeypress>,
    modifiers: Vec<u16>,
    key: String,
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
        }
    }

    #[inline]
    pub fn modifier(&self) -> u16 {
        self.modifiers.iter().fold(0, |acc, &m| acc | m)
    }

    /// TODO: grab compound keys... REFACTOR
    pub fn keycode(&self, sym: &keysyms::KeySymbols) -> Result<u8, String> {
        let keysym = unsafe {
            let c_str = CString::new(self.key.to_owned()).map_err(|e| e.to_string())?;
            x11::xlib::XStringToKeysym(c_str.as_ptr()) as u32
        };

        match sym.get_keycode(keysym).next() {
            Some(code) => Ok(code),
            None => Err(format!("Keycode for \"{}\" not found.", self.key).to_owned()),
        }
    }
}

impl Clone for OnKeypress {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            modifiers: self.modifiers.clone(),
            callback: dyn_clone::clone_box(&*self.callback),
        }
    }
}

impl OnKeypress {
    #[inline]
    pub fn call(&self, ctx: EventContext) -> Result<(), String> {
        self.callback.call(ctx)
    }
}
