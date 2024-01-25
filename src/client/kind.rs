use xcb_util::ewmh;

use crate::client::{
    Client,
    ClientAction,
    ClientState,
};

#[derive(Clone, PartialEq, Debug)]
pub enum ClientType {
    Normal,
    Dock,
}

impl Client {
    /// Sets the client type and performs additional configurations if needed.
    pub fn set_type(&mut self, conn: &ewmh::Connection, r#type: ClientType) {
        self.r#type = r#type;
        match self.r#type {
            // Docks don't need to be managed; they are sticky and don't have a specified tag.
            ClientType::Dock => {
                self.is_controlled = false;
                self.add_state(conn, ClientState::Sticky);
            },
            _ => {
                self.enable_event_mask(&conn);
                self.allow_actions(
                    conn,
                    vec![
                        ClientAction::Maximize,
                        ClientAction::Fullscreen,
                        ClientAction::ChangeTag,
                        ClientAction::Resize,
                        ClientAction::Move,
                    ],
                );
            },
        }
    }
}

