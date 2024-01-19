use core::fmt;
use std::sync::Arc;

use xcb_util::ewmh;

use crate::{util, clients::ClientID, event_context::EventContext};

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
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the event was successfully registered, or `Err` if the event is
    /// already being listened to.
    fn listen_event(&mut self, e: MouseEvent) -> Result<(), String> {
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
    
        self.events.push(e);

        Ok(())
    }

    pub fn on(&mut self, e: MouseEvent, callback: Box<dyn FnOnClick>) {
        if !self.has_event(&e) {
            _ = self.listen_event(e);
        }

        self.on_click.push(dyn_clone::clone_box(&*callback))
    }

    pub fn trigger_with(&self, e: MouseEvent, ctx: EventContext, info: MouseInfo) -> Result<(), String> {
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

pub trait FnOnClick: dyn_clone::DynClone {
    fn call(&self, ctx: EventContext, info: MouseInfo) -> Result<(), String>;
}

impl<F> FnOnClick for F
where 
    F: Fn(EventContext, MouseInfo) -> Result<(), String>  + Clone
{
    fn call(&self, ctx: EventContext, info: MouseInfo) -> Result<(), String> {
        self(ctx, info)
    }
}

/// Represents information about mouse in events.
#[derive(Clone)]
pub struct MouseInfo {
    /// The client's ID where the mouse was pressed.
    pub c_id: ClientID,

    /// The x position of where the mouse was pressed. 0 is top-left.
    pub x: i16,

    /// The y position of where the mouse was pressed. 0 is top-left.
    pub y: i16,

    /// The mask of modifiers when the mouse was pressed. For example:
    /// ```
    /// // When pressing Mouse + Shift
    /// assert_eq!(modifier, 1);
    ///
    /// // When pressing Mouse + Shift + Ctrl
    /// assert_eq!(modifier, 1 | 4);
    /// ```
    ///
    /// You can also use `util::modkeys` to get the modifiers constants.
    pub modifier: u16,
}

impl MouseInfo {
    /// Creates a new `MouseInfo`. `Pos` is a tuple with (x, y) order.
    pub fn new(c_id: ClientID, modifier: u16, pos: (i16, i16)) -> Self {
        Self {
            c_id,
            x: pos.0,
            y: pos.1,
            modifier,
        }
    }
}
