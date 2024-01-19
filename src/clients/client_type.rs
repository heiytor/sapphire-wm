use xcb_util::ewmh;

use crate::{
    clients::{
        Client,
        client_state::ClientState,
    },
    tag::TagID,
};

use super::client_action::ClientAction;

#[derive(Clone, PartialEq, Debug)]
pub enum ClientType {
    Normal,
    Dock,
}

impl Client {
    /// Gets the type of the client.
    pub fn get_type(&self) -> &ClientType {
        &self.r#type
    }

    /// Sets the client type and performs additional configurations if needed.
    pub fn set_type(&mut self, conn: &ewmh::Connection, r#type: ClientType, tag_id: TagID) {
        self.r#type = r#type;
        match self.r#type {
            ClientType::Dock => {
                self.is_controlled = false;
                self.add_state(conn, ClientState::Sticky);
                self.set_tag(conn, 0xFFFFFF);
            },
            _ => {
                self.enable_event_mask(&conn);
                self.set_tag(conn, tag_id);
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

