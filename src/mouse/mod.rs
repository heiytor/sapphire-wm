mod callback;

use std::sync::Arc;

use xcb_util::ewmh;

use crate::{
    util,
    event::{
        EventContext,
        MouseEvent,
    },
    errors::Error,
};

pub use crate::mouse::callback::{
    FnOnClick,  
    MouseInfo,
};

pub struct Mouse {
    conn: Arc<ewmh::Connection>,
    events: Vec<MouseEvent>,
    on_click: Vec<Box<dyn FnOnClick>>,
}

impl Mouse {
    pub fn new(conn: Arc<ewmh::Connection>) -> Self {
        // Disables implicit sloppy focus.
        xcb::set_input_focus(
            &conn,
            xcb::INPUT_FOCUS_PARENT as u8,
            // The input focus needs to be the root window to avoid keyboard blocks.
            xcb::INPUT_FOCUS_POINTER_ROOT,
            xcb::CURRENT_TIME,
        );

        Self {
            conn,
            events: vec![],
            on_click: vec![],
        }
    }
}

impl Mouse {
    /// Verifies whether the window manager is already listening for event `e`.
   fn has_event(&self, e: &MouseEvent) -> bool {
        self.events.iter().any(|me| me == e)
    }

    /// Listens for the specified mouse event and configures the window manager accordingly.
    fn listen_event(&mut self, e: MouseEvent) {
        if self.has_event(&e) {
            return
        }

        match e {
            MouseEvent::Click => {
                xcb::grab_button(
                    &self.conn,
                    false,
                    util::get_screen(&self.conn).root(),
                    xcb::EVENT_MASK_BUTTON_RELEASE as u16,
                    xcb::GRAB_MODE_SYNC as u8,
                    xcb::GRAB_MODE_ASYNC as u8,
                    xcb::NONE,
                    xcb::NONE,
                    1,
                    xcb::MOD_MASK_ANY as u16,
                );
            }
        };
    
        self.events.push(e);
    }

    /// Register a callback `cb` to be executed when the event `e` is triggered.
    pub fn on(&mut self, e: MouseEvent, cb: Box<dyn FnOnClick>) {
        if !self.has_event(&e) {
            self.listen_event(e);
        }

        self.on_click.push(dyn_clone::clone_box(&*cb));
    }

    /// Triggers the event `e` with the provided context and information.
    pub fn trigger_with(&self, e: MouseEvent, ctx: EventContext, info: MouseInfo) -> Result<(), Error> {
        match e {
            MouseEvent::Click => {
                for cb in self.on_click.iter() {
                    cb.call(ctx.clone(), info.clone())?;
                }
            },
        }

        Ok(())
    }
}
