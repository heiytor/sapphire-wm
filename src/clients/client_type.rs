use xcb_util::ewmh;

use crate::{
    clients::{
        Client,
        client_state::ClientState,
    },
    tag::TagID, util,
};

use super::client_action::ClientAction;

#[derive(Clone, PartialEq, Debug)]
pub enum ClientType {
    Normal,
    Dock,
}

impl Client {
    /// Sets the client type and performs additional configurations if needed.
    pub fn set_type(&mut self, conn: &ewmh::Connection, r#type: ClientType, tag_id: TagID) {
        self.r#type = r#type;
        match self.r#type {
            // Docks don't need to be managed; they are sticky and don't have a specified tag.
            ClientType::Dock => {
                self.is_controlled = false;
                self.add_state(conn, ClientState::Sticky);
                util::set_client_tag(conn, self.wid, 0xFFFFFF);
            },
            _ => {
                util::set_client_tag(conn, self.wid, tag_id);
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

