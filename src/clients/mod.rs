pub mod client_action;
pub mod client_state;
pub mod client_type;

use xcb_util::ewmh;

use crate::{clients::{
    client_action::ClientAction,
    client_state::ClientState,
    client_type::ClientType,
}, util::Operation};

pub type WindowID = u32;

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

    /// Verifies if the client allows the specified action `a`.
    pub fn allows_action(&self, a: &ClientAction) -> bool {
        self.allowed_actions.iter().any(|ca| ca == a)
    }

    /// Adds the specified `action` to the client's list of allowed list if it is not already
    /// present. It also updates the corresponding `_NET_WM_ALLOWD_ACTIONS` property to reflect
    /// the updated list of actions.
    /// 
    /// If you need to add `n` actions, use `Client::allow_actions` instead.
    pub fn allow_action(&mut self, conn: &ewmh::Connection, action: ClientAction) {
        if self.allows_action(&action) {
            return
        }

        let new_net_allowed_actions: Vec<u32> = self.allowed_actions
            .iter()
            .flat_map(|s| s._net_wm_allowed_actions(conn))
            .collect();

        ewmh::set_wm_allowed_actions(&conn, self.wid, new_net_allowed_actions.as_slice());
    }

    /// Similar to `Client::allow_action`, but allows `n` actions at a time. Since each
    /// `allowed_actions[i]` must be unique, it iterates over the provided actions, removing
    /// those that are already allowed.
    pub fn allow_actions(&mut self, conn: &ewmh::Connection, actions: Vec<ClientAction>) {
        let new_actions: Vec<ClientAction> = actions
            .into_iter()
            .filter(|action| !self.allows_action(action))
            .collect();

        self.allowed_actions.extend(new_actions);

        let new_net_allowed_actions: Vec<u32> = self.allowed_actions
            .iter()
            .flat_map(|s| s._net_wm_allowed_actions(conn))
            .collect();

        ewmh::set_wm_allowed_actions(&conn, self.wid, new_net_allowed_actions.as_slice());
    }

    /// Verifies if the client has the specified state `s`.
    pub fn has_state(&self, s: &ClientState) -> bool {
        self.states.iter().any(|cs| cs == s)
    }

    /// Returns the last state of the client. As the latest pushed states have more privileges
    /// when the window manager needs to perform actions related to the client's state,
    /// use this function to determine which client's state to handle. When the client
    /// does not have any state, it returns `ClientState::Tile`.
    pub fn last_state(&self) -> &ClientState {
        self.states.last().unwrap_or(&ClientState::Tile)
    }

    /// Adds the specified `state` to the client's list of states if it is not already present. It
    /// also updates the corresponding `xcb::WM_STATE` property to reflect the updated list of states.
    pub fn add_state(&mut self, conn: &ewmh::Connection, state: ClientState) {
        if self.has_state(&state) {
            return;
        }

        xcb::change_property(
            conn,
            xcb::PROP_MODE_APPEND as u8,
            self.wid,
            conn.WM_STATE(),
            xcb::ATOM_ATOM,
            32,
            state._net_wm_state(conn).as_slice(),
        );

        self.states.push(state);
    }

    /// Removes the specified `state` from the client's list of states if it is present. It also
    /// updates the corresponding `xcb::WM_STATE` property to reflect the updated list of states.
    pub fn remove_state(&mut self, conn: &ewmh::Connection, state: ClientState) {
        if !self.has_state(&state) {
            return;
        }

        self.states.retain(|s| s != &state);

        let new_net_wm_state: Vec<u32> = self.states
            .iter()
            .flat_map(|s| s._net_wm_state(conn))
            .collect();

        xcb::change_property(
            conn,
            xcb::PROP_MODE_REPLACE as u8,
            self.wid,
            conn.WM_STATE(),
            xcb::ATOM_ATOM,
            32,
            new_net_wm_state.as_slice(),
        );
    }

    pub fn set_state(&mut self, conn: &ewmh::Connection, state: ClientState, operation: Operation) -> Result<(), String> {
        match operation {
            Operation::Add => self.add_state(conn, state),
            Operation::Remove => self.remove_state(conn, state),
            Operation::Toggle => {
                if self.has_state(&state) {
                    self.remove_state(conn, state)
                } else {
                    self.add_state(conn, state)
                }
            },
            Operation::Unknown => return Err("Unknown operation".to_owned()),
        }

        Ok(())
    }

    /// Gets the type of the client.
    pub fn get_type(&self) -> &ClientType {
        &self.r#type
    }

    /// Sets the client type and performs additional configurations if needed.
    pub fn set_type(&mut self, r#type: ClientType) {
        self.r#type = r#type;
        match self.r#type {
            ClientType::Dock => {
                self.is_controlled = false;
            },
            _ => {},
        }
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
