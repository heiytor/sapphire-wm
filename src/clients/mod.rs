pub mod client_action;
pub mod client_state;
pub mod client_type;

use xcb_util::ewmh;

use crate::clients::{
    client_action::ClientAction,
    client_state::ClientState,
    client_type::ClientType,
};

pub type WindowID = u32;

#[derive(Clone)]
pub struct Client {
    pub wid: WindowID,
    pub pid: u32,

    is_controlled: bool,
    is_visible: bool,

    // TODO: docs
    r#type: ClientType,
    
    /// Represents the list of current `xcb::WM_STATE` atoms of the client.
    /// Each state must be unique in the vector.
    ///
    /// The importance of states is from last to first, as the latest pushed states
    /// are treated with more privileges. For example, a client with `states` equal to
    /// `[ClientState::Fullscreen, ClientState::Maximized]` must be drawn as maximized.
    /// When removing `ClientState::Maximized` from the list, the client must be drawn as fullscreen.
    ///
    /// Some functions that returns the state may sometimes return `ClientState::Tile`. This state
    /// is special and is never included in the list; it simply indicates that the client
    /// doesn't have any configured state.
    ///
    /// Refer to: https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm46201142858672
    states: Vec<ClientState>,

    /// Represents the list of current `_NET_ALLOWED_ACTIONS` atoms of the client.
    /// Each action must be unique in the vector.
    ///
    /// Refer to: https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm46201142837824
    allowed_actions: Vec<ClientAction>,

    pub padding_top: u32,
    pub padding_bottom: u32,
    pub padding_left: u32,
    pub padding_right: u32,

    pub tag: u32,
    pub screen: u32,
    pub is_focused: bool,
}

impl Default for Client {
    fn default() -> Self {
        Client { 
            pid: 0,
            wid: 0,
            is_controlled: false,
            padding_top: 0,
            padding_bottom: 0,
            padding_left: 0,
            padding_right: 0,
            is_visible: false,
            r#type: ClientType::Normal,
            tag: 0,
            screen: 0,
            is_focused: false,
            states: vec![],
            allowed_actions: vec![],
        }
    }
}

impl Client {
    pub fn new(wid: WindowID) -> Self {
        Client { 
            wid,
            pid: 0,
            is_controlled: true,
            padding_top: 0,
            padding_bottom: 0,
            padding_left: 0,
            padding_right: 0,
            is_visible: true,
            r#type: ClientType::Normal,
            tag: 0,
            screen: 0,
            is_focused: false,
            states: vec![],
            allowed_actions: vec![],
        }
    }

    pub fn show(&mut self, conn: &ewmh::Connection) {
        self.is_visible = true;
        xcb::map_window(conn, self.wid);
    }

    pub fn hide(&mut self, conn: &ewmh::Connection) {
        self.is_visible = false;
        xcb::unmap_window(conn, self.wid);
    }

    /// Sets the padding values for the client.
    pub fn set_paddings(&mut self, top: u32, bottom: u32, left: u32, right: u32) {
        self.padding_top = top;
        self.padding_bottom = bottom;
        self.padding_left = left;
        self.padding_right = right;
    }

    /// Returns whether the client needs control.
    #[inline]
    pub fn is_controlled(&self) -> bool {
        self.is_controlled
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }


    pub fn set_border(&self, conn: &ewmh::Connection, color: u32) {
        xcb::change_window_attributes(
            conn,
            self.wid,
            &[(xcb::CW_BORDER_PIXEL, color)],
        );
    }

    pub fn set_input_focus(&self, conn: &ewmh::Connection) {
        xcb::set_input_focus(
            conn,
            xcb::INPUT_FOCUS_PARENT as u8,
            self.wid,
            xcb::CURRENT_TIME
        );
    }

    /// Sends a destroy notification to the window manager with the client's window ID.
    pub fn kill(&self, conn: &ewmh::Connection) {
        xcb::destroy_window(conn, self.wid);
    }

    /// TODO: rename it!
    pub fn enable_event_mask(&self, conn: &ewmh::Connection) {
        xcb::change_window_attributes(
            conn,
            self.wid,
            &[(
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_PROPERTY_CHANGE
                | xcb::EVENT_MASK_STRUCTURE_NOTIFY,
            )],
        );
    }
}
