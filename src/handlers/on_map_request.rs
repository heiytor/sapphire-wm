use std::sync::{Arc, Mutex};

use xcb_util::ewmh;

use crate::{clients::{client::{Client, ClientType}, clients::Clients}, util};


pub fn handle(event: &xcb::MapRequestEvent, conn: &ewmh::Connection, clients: Arc<Mutex<Clients>>) {
    xcb::map_window(&conn, event.window());

    {
        let mut clients = clients.lock().unwrap();

        if clients.contains(event.window()) {
            return;
        }

        let mut client = Client::new(event.window());

        if util::window_has_type(conn, event.window(), conn.WM_WINDOW_TYPE_DOCK()) {
            client.set_type(ClientType::Dock);
        }

        if let Ok (strut) = ewmh::get_wm_strut_partial(&conn, event.window()).get_reply() {
            client.set_paddings(strut.top, strut.bottom, strut.left, strut.right);
        };

        let r#type = client.get_type();
        if r#type != &ClientType::Dock {
            client.set_inactive_border(conn);
        }

        clients.manage(client);
    };

    conn.flush();
}
