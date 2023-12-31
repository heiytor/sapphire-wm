use std::sync::{Arc, Mutex};

use xcb_util::ewmh;

use crate::clients::clients::Clients;


pub fn handle(event: &xcb::DestroyNotifyEvent, conn: &ewmh::Connection, clients: Arc<Mutex<Clients>>) {
    {
        // TODO: handle errors
        let mut clients = clients.lock().unwrap();
        clients.unmanage(event.window());
    };

    conn.flush();
}
