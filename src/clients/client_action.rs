use xcb_util::ewmh;

use crate::clients::Client;

#[derive(Clone, PartialEq, Debug)]
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

impl Client {
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

        ewmh::set_wm_allowed_actions(&conn, self.id, new_net_allowed_actions.as_slice());
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

        ewmh::set_wm_allowed_actions(&conn, self.id, new_net_allowed_actions.as_slice());
    }
}
