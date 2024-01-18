use core::fmt;
use std::sync::Arc;

use xcb_util::ewmh;

use crate::util;

/// Represents the events that the window manager should listen for mouse actions.
#[derive(PartialEq)]
pub enum MouseEvent {
    /// Represents the `xcb::EVENT_MASK_BUTTON_PRESS` mask, which is globally grabbed on the `screen.root()`
    /// without any modifiers. It sends an `xcb::BUTTON_PRESS` event and is used to set focus on the window when clicked.
    /// This event blocks all other clients from receiving mouse events, and the window manager
    /// should allow the `xcb::ALLOW_REPLAY_POINTER` event to release it.
    ///
    /// TODO:
    /// Change the event mask to `xcb::EVENT_MASK_BUTTON_RELEASE`
    Click,
}

impl fmt::Display for MouseEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MouseEvent::Click => write![f, "click mode"],
        }
    }
}

pub struct Mouse {
    conn: Arc<ewmh::Connection>,
    focus_modes: Vec<MouseEvent>,
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
            focus_modes: vec![],
        }
    }
}

impl Mouse {
    /// Verifies whether the window manager is already listening for event `e`.
    pub fn has_event(&self, e: &MouseEvent) -> bool {
        self.focus_modes.iter().any(|me| me == e)
    }

    /// Listens for the specified mouse event and configures the window manager accordingly.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the event was successfully registered, or `Err` if the event is
    /// already being listened to.
    pub fn listen_event(&mut self, e: MouseEvent) -> Result<(), String> {
        if self.has_event(&e) {
            return Err(format!["mouse already has {}", e])
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
    
        self.focus_modes.push(e);

        Ok(())
    }
}
