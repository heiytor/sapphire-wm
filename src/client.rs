use std::{sync::Arc, collections::VecDeque};

use xcb_util::ewmh;

pub struct Client {
    wid: u32,
}

impl Client {
    pub fn new(wid: u32) -> Self {
        Client { wid }
    }
}

pub struct Config {
    /// Windows border in pixels.
    pub border: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config { border: 0 }
    }
}

pub struct Clients {
    /// EWMH connection.
    conn: Arc<ewmh::Connection>,
    /// Currently managed X windows.
    clients: VecDeque<Client>,

    pub config: Config,
}

impl Clients {
    pub fn new(conn: Arc<ewmh::Connection>) -> Self {
        Clients {
            conn,
            clients: VecDeque::new(),
            config: Config::default(),
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
        self.clients.push_front(client);
    }

    /// Unmanages an X Window. Removes it from the "_NET_CLIENT_LIST", and if the window
    /// is the "_NET_ACTIVE_WINDOW", sets the last created window as active.
    pub fn unmanage(&mut self, wid: u32) {
        self.refresh_client_list();
        self.clients.retain(|c| c.wid != wid);

        let last_wid: Option<u32> = self.clients.front().map(|c| c.wid);
        let last_wid: u32 = last_wid.unwrap_or(xcb::WINDOW_NONE);
        self.set_active(last_wid);
    }

    pub fn resize_tiles(&self, screen: xcb::Screen) {
        // ....
        let screen_w = screen.width_in_pixels() as u32;
        let screen_h = screen.height_in_pixels() as u32;

        // Starting tilling at top-right
        let mut window_x: u32 = 0;
        let mut window_y: u32 = 0;

        // ...
        let mut window_h: u32 = screen_h - self.config.border * 2;
        let mut window_w: u32 = if self.clients.len() == 1 { 
            screen_w - self.config.border * 2
        } else { 
            screen_w / 2 - self.config.border
        };

        for (i, client) in self.clients.iter().enumerate() {
            if i > 0 {
                // Since the master window always fills the left-middle of the
                // screen, the other windows will only occupy the right-middle portion.
                window_w = (screen_w / 2) - self.config.border * 2;
                window_x = screen_w / 2;

                // Adjusting the height for each window located in the right-middle portion of the screen
                // to ensure they fit proportionally based on the total number of windows.
                let height_per_window = screen_h / (self.clients.len() - 1) as u32;

                window_y = height_per_window * (i - 1) as u32;
                window_h = if client.wid == self.clients.back().unwrap().wid {
                    height_per_window - (self.config.border * 2)
                } else {
                    height_per_window - self.config.border
                };
            }

            xcb::configure_window(
                &self.conn,
                client.wid,
                &[
                    (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, self.config.border),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, window_h),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, window_w),
                    (xcb::CONFIG_WINDOW_X as u16, window_x),
                    (xcb::CONFIG_WINDOW_Y as u16, window_y),
                ],
            );
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
