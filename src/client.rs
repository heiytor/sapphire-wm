use std::{sync::Arc, collections::VecDeque};

use xcb_util::ewmh;

use crate::util;

pub struct Client {
    wid: u32,
    is_fullscreen: bool,
}

impl Client {
    pub fn new(wid: u32) -> Self {
        Client { 
            wid,
            is_fullscreen: false,
        }
    }
}

pub struct Config {
    /// Windows border in pixels.
    pub border: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config { border: 8 }
    }
}

pub struct Clients {
    pub config: Config,

    /// EWMH connection.
    conn: Arc<ewmh::Connection>,

    /// Currently managed X windows.
    clients: VecDeque<Client>,

    // TODO: Each screen have its own active_client, and probably each workspace as well. Maybe this can
    // be a HashMap where the key is the screen, and the value is an array of wid, with each index
    // representing the workspace.
    //
    /// Index of the current active client among the managed clients.
    active_client: usize,
}

impl Clients {
    pub fn new(conn: Arc<ewmh::Connection>) -> Self {
        Clients {
            conn,
            clients: VecDeque::new(),
            config: Config::default(),
            active_client: 0,
        }
    }
}

impl Clients {
    /// Adds a new X Window that the window manager should manage. Updates the
    /// "_NET_CLIENT_LIST" to include the created window and sets it as the
    /// "_NET_ACTIVE_WINDOW".
    pub fn manage(&mut self, client: Client) {
        // TODO.
        let client_wid = client.wid;

        self.clients.push_front(client);
        self.set_active(client_wid);
        self.refresh_client_list();
    }

    /// Unmanages an X Window. Removes it from the "_NET_CLIENT_LIST", and if the window
    /// is the "_NET_ACTIVE_WINDOW", sets the last created window as active.
    pub fn unmanage(&mut self, wid: u32) {
        self.clients.retain(|c| c.wid != wid);

        let last_wid: Option<u32> = self.clients.front().map(|c| c.wid);
        let last_wid: u32 = last_wid.unwrap_or(xcb::WINDOW_NONE);

        self.set_active(last_wid);
        self.refresh_client_list();
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

        for (i, client) in self.clients.iter().filter(|c| !c.is_fullscreen).enumerate() {
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

        // Fullscreen windows
        for client in self.clients.iter().filter(|c| c.is_fullscreen) {
            xcb::configure_window(
                &self.conn,
                client.wid,
                &[
                    (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, 0),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, screen_h),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, screen_w),
                    (xcb::CONFIG_WINDOW_X as u16, 0),
                    (xcb::CONFIG_WINDOW_Y as u16, 0),
                ],
            );
        }

        self.conn.flush();
    }
}

pub enum Dir {
    Left,
    Right,
}

const MASTER_CLIENT: usize = 0; 

impl Clients {
    /// Swaps the master client to the active client. If the active client is already the
    /// master, do nothing.
    pub fn swap_master(&mut self) {
        if self.active_client == 0 {
            return
        }

        self.clients.swap(MASTER_CLIENT, self.active_client);
        self.set_active(self.clients[MASTER_CLIENT].wid);
        self.resize_tiles(util::get_screen(&self.conn));
    }

    pub fn move_focus(&mut self, dir: Dir) {
        let (idx, default_idx) = match dir {
            Dir::Right => (self.active_client + 1, 0),
            Dir::Left => {
                (
                    self.active_client.
                        checked_sub(1).
                        unwrap_or_else(|| self.clients.len() - 1),
                    0,
                )
            }
        };

        let wid: u32 = self.clients.get(idx).map_or_else(
            || self.clients[default_idx].wid,
            |client| client.wid,
        );

        self.set_active(wid);
    }

    // TODO: error handling
    pub fn toggle_fullscreen(&mut self) {
        let active_client = &mut self.clients[self.active_client];

        active_client.is_fullscreen = !active_client.is_fullscreen;

        let data = if active_client.is_fullscreen {
            self.conn.WM_STATE_FULLSCREEN()
        } else {
            0
        };

        xcb::change_property(
            &self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            active_client.wid,
            self.conn.WM_STATE(),
            xcb::ATOM_ATOM,
            32,
            &[data],
        );

        self.resize_tiles(util::get_screen(&self.conn));
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

    /// Sets the active window to the specified window ID. Configure the "_NET_ACTIVE_WINDOW" and
    /// sets the input focus.
    #[inline]
    pub(self) fn set_active(&mut self, wid: u32) {
        if let Some(idx) = self.clients.iter().position(|c| c.wid == wid) {
            self.active_client = idx;
            ewmh::set_active_window(&self.conn, 0, wid);
            xcb::set_input_focus(
                &self.conn,
                xcb::INPUT_FOCUS_PARENT as u8,
                wid,
                xcb::CURRENT_TIME,
            );
        }
    }
}
