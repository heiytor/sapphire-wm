use std::sync::{Arc, Mutex};

use xcb_util::ewmh;

use crate::{client::{Client, Clients}, util};

pub fn handle(event: &xcb::MapRequestEvent, conn: &ewmh::Connection, clients: Arc<Mutex<Clients>>) {
    xcb::map_window(&conn, event.window());

    {
        let mut clients = clients.lock().unwrap();

        if clients.contains(event.window()) {
            return;
        }

        let mut client = Client::new(event.window());

        if util::client_has_type(conn, event.window(), conn.WM_WINDOW_TYPE_DOCK()) {
            client.set_docker();
            client.remove_controll();
        }

        if let Ok (strut) = ewmh::get_wm_strut_partial(&conn, event.window()).get_reply() {
            client.padding_top = strut.top;
            client.padding_bottom = strut.bottom;
            client.padding_left = strut.left;
            client.padding_right = strut.right;
        };

        clients.manage(client);
        clients.resize_tiles(util::get_screen(&conn));
    };
}
