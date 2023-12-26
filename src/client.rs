use std::sync::Arc;

use xcb_util::ewmh;

pub struct Client {
    wid: u32,
}

impl Client {
    pub fn new(wid: u32) -> Self {
        Client { wid }
    }
}

pub struct Clients {
    /// EWMH connection.
    conn: Arc<ewmh::Connection>,
    /// Currently managed X windows.
    clients: Vec<Client>,
}

impl Clients {
    pub fn new(conn: Arc<ewmh::Connection>) -> Self {
        Clients {
            conn,
            clients: Vec::new(),
        }
    }
}

impl Clients {
    /// Adds a new X Window that the window manager should manage. Updates the
    /// "_NET_CLIENT_LIST" to include the created window and sets it as the
    /// "_NET_ACTIVE_WINDOW".
    pub fn manage(&mut self, client: Client) {
        self.refresh_client_list();
        self.set_active(client.wid);
        self.clients.push(client);
    }

    /// Unmanages an X Window. Removes it from the "_NET_CLIENT_LIST", and if the window
    /// is the "_NET_ACTIVE_WINDOW", sets the last created window as active.
    pub fn unmanage(&mut self, wid: u32) {
        self.refresh_client_list();
        self.clients.retain(|c| c.wid != wid);

        let last_wid: Option<u32> = self.clients.last().map(|c| c.wid);
        let last_wid: u32 = last_wid.unwrap_or(xcb::WINDOW_NONE);
        self.set_active(last_wid);
    }

    pub fn resize_tiles(&self, screen: xcb::Screen) {
        let mut x = 0;
        let y = 0;

        if self.clients.len() == 1 {
            xcb::configure_window(
                &self.conn,
                self.clients[0].wid,
                &[
                    (xcb::CONFIG_WINDOW_X as u16, x as u32),
                    (xcb::CONFIG_WINDOW_Y as u16, y as u32),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, screen.width_in_pixels() as u32),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, screen.height_in_pixels() as u32),
                ],
            );

            return
        }

        for client in self.clients.iter() {
            // do something

            xcb::configure_window(
                &self.conn,
                client.wid,
                &[
                    (xcb::CONFIG_WINDOW_X as u16, x as u32),
                    (xcb::CONFIG_WINDOW_Y as u16, y as u32),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, (screen.width_in_pixels() / 2) as u32),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, (screen.height_in_pixels()/*  / 2 */) as u32),
                ],
            );
            
            x += screen.width_in_pixels() / 2;
        }
    }
}

impl Clients {
    /// Refreshes the "_NET_CLIENT_LIST" with the current list of clients.
    #[inline]
    pub(self) fn refresh_client_list(&self) {
        ewmh::set_client_list(
            &self.conn,
            0,
            &self.clients.iter().map(|c| c.wid).collect::<Vec<u32>>(),
        );
    }

    /// Sets the active window to the specified window ID.
    #[inline]
    pub(self) fn set_active(&self, wid: u32) {
        ewmh::set_active_window(&self.conn, 0, wid);
    }
}
