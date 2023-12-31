use std::sync::{Arc, Mutex};

use xcb_util::ewmh;

use crate::{clients::clients::Clients, util::{self, Operation}};

pub fn handle(event: &xcb::ClientMessageEvent, conn: &ewmh::Connection, clients: Arc<Mutex<Clients>>) {
    if event.type_() == conn.WM_STATE() {
        // SEE:
        // > https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm46201142858672
        let data = event.data().data32();

        let action = match data[0] {
            ewmh::STATE_ADD => Operation::Add,
            ewmh::STATE_REMOVE => Operation::Remove,
            ewmh::STATE_TOGGLE => Operation::Toggle,
            _ => Operation::Unknown,
        };
        let property = data[1];

        {
            let mut clients = clients.lock().unwrap();
            if property == conn.WM_STATE_FULLSCREEN() {
                _ = clients
                    .set_fullscreen(event.window(), action)
                    .map_err(|e| util::notify_error(e));
            }
        };
    }

    conn.flush();
}

