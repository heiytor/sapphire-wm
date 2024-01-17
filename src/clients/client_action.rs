use xcb_util::ewmh;

#[derive(PartialEq, Debug)]
pub enum ClientAction {
    Fullscreen,
    Maximize,
    ChangeTag,
    Resize,
    Move,
    Close,
}

impl ClientAction {
    /// Converts the state to its `xcb::WM_STATE` atom. It returns a vector of atoms
    /// since a state can have `n` atom representations, as in the case of
    /// `ClientState::Maximized`.
    pub fn _net_wm_allowed_actions(&self, conn: &ewmh::Connection) -> Vec<u32> {
        match self {
            ClientAction::Fullscreen => vec![conn.WM_ACTION_FULLSCREEN()],
            ClientAction::ChangeTag => vec![conn.WM_ACTION_CHANGE_DESKTOP()],
            ClientAction::Maximize => vec![
                conn.WM_ACTION_MAXIMIZE_VERT(),
                conn.WM_ACTION_MAXIMIZE_HORZ(),
            ],
            ClientAction::Resize => vec![conn.WM_ACTION_RESIZE()],
            ClientAction::Close => vec![conn.WM_ACTION_CLOSE()],
            ClientAction::Move => vec![conn.WM_ACTION_MOVE()],
        }
    }
}

