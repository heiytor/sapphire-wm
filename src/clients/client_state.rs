use xcb_util::ewmh;

/// Represents the `xcb::WM_STATE` atom.
#[derive(PartialEq, Debug)]
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
            ClientState::Tile => vec![0], // When tiling, the client doesn't have any WM state.
        }
    }
}
