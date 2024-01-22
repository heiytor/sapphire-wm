use xcb_util::ewmh;

use crate::{
    clients::Client,
    util::Operation,
};

/// Represents the `xcb::WM_STATE` atom.
#[derive(Clone, PartialEq, Debug)]
pub enum ClientState {
    /// Indicates that a client does not have any specific state.
    Tile,

    /// Indicates that a client has the `WM_STATE_FULLSCREEN` atom.
    Fullscreen,

    /// Indicates that a client has both the `WM_STATE_MAXIMIZED_VERT` and
    /// `WM_STATE_MAXIMIZED_HORZ` atoms. SapphireWM does not allow a client to have
    /// just one of these atoms at a time, so we ensure that they are both seted together.
    Maximized,

    /// Indicates that a client has the `WM_STATE_STICKY` atom. This state cannot be
    /// toggled, as SapphireWM only allows docks to have it.
    Sticky,

    /// Indicates that a client has the `_NET_WM_STATE_HIDDEN` atom. 
    Hidden,
}

impl ClientState {
    /// Converts the state to its `xcb::WM_STATE` atom. It returns a vector of atoms
    /// since a state can have `n` atom representations, as in the case of
    /// `ClientState::Maximized`.
    pub fn _net_wm_state(&self, conn: &ewmh::Connection) -> Vec<u32> {
        match self {
            ClientState::Fullscreen => vec![conn.WM_STATE_FULLSCREEN()],
            ClientState::Maximized => vec![
                conn.WM_STATE_MAXIMIZED_VERT(),
                conn.WM_STATE_MAXIMIZED_HORZ(),
            ],
            ClientState::Sticky => vec![conn.WM_STATE_STICKY()],
            ClientState::Hidden => vec![conn.WM_STATE_HIDDEN()],
            ClientState::Tile => vec![0], // When tiling, the client doesn't have any WM state.
        }
    }
}

impl Client {
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
            self.id,
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
            self.id,
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
}
