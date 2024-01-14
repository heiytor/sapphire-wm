use std::{sync::Arc, collections::VecDeque};

use xcb_util::ewmh;

use crate::{util::{self, Operation}, clients::client::ClientState, config::Config};

use super::client::{Client, ClientType};


pub struct Clients {
    config: Arc<Config>,
    conn: Arc<ewmh::Connection>,

    /// Currently managed X windows.
    clients: VecDeque<Client>,

    // TODO: Each screen have its own active_client, and probably each workspace as well. Maybe this can
    // be a HashMap where the key is the screen, and the value is an array of wid, with each index
    // representing the workspace.
    //
    /// Index of the current active client among the managed clients.
    active_client: usize,

    pub active_desktop: u32,
}

impl Clients {
    pub fn new(conn: Arc<ewmh::Connection>, config: Arc<Config>) -> Self {
        Clients {
            conn,
            active_desktop: 0,
            config,
            clients: VecDeque::new(),
            active_client: 0,
        }
    }
}

pub enum Dir {
    Left,
    Right,
}

pub const MASTER_CLIENT: usize = 0; 

#[allow(dead_code)]
impl Clients {
    /// Adds a new X window that the window manager should managea and updates the
    /// `_NET_CLIENT_LIST` to include the created window.
    pub fn manage(&mut self, client: Client) {
        self.clients.push_front(client);
        self.refresh_client_list();
    }

    /// Unmanages an X window and removes it from the `_NET_CLIENT_LIST`.
    pub fn unmanage(&mut self, wid: u32) {
        self.clients.retain(|c| c.wid != wid);
        self.refresh_client_list();
    }

    /// Retrieves a client with the specified window ID.
    #[inline]
    pub fn get(&self, wid: u32) -> Option<&Client> {
        self.clients
            .iter()
            .find(|c| c.wid == wid)
    }

    /// Retrieves a client with the specified window ID as mut.
    #[inline]
    pub fn get_mut(&mut self, wid: u32) -> Option<&mut Client> {
        self.clients
            .iter_mut()
            .find(|c| c.wid == wid)
    }

    /// Retrieves the focused client on screen `s` and tag `t`.
    #[inline]
    pub fn get_focused(&self, s: u32, t: u32) -> Option<&Client> {
        self.clients
            .iter()
            .find(|c| c.is_focused && c.screen == s && c.tag == t)
    }

    pub fn set_focused(&mut self, client: &Client) {
        let set_border_color = |wid: u32, color: u32| {
            xcb::change_window_attributes(
                &self.conn,
                wid,
                &[(xcb::CW_BORDER_PIXEL, color)],
            );
        };

        // Unfocus the old focused client if any. TODO
        // if let Some(c) = self.get_focused_mut(client.screen, client.tag) {
        if let Some(c) = self.clients.iter_mut().find(|c| c.is_focused && c.screen == client.screen && c.tag == client.tag) {
            set_border_color(c.wid, self.config.border.inactive_color);
        }

        set_border_color(client.wid, self.config.border.active_color);

        xcb::set_input_focus(
            &self.conn,
            xcb::INPUT_FOCUS_PARENT as u8,
            client.wid,
            xcb::CURRENT_TIME,
        );
    }

    /// Swaps the master client to the active client. If the active client is already the
    /// master, do nothing.
    pub fn swap_master(&mut self) {
        if self.active_client == 0 {
            return
        }

        self.clients.swap(MASTER_CLIENT, self.active_client);
        // self.set_focused(&mut self.clients[MASTER_CLIENT]);
        self.resize_tiles(util::get_screen(&self.conn));
    }

    /// Moves the focus to the next client in the specified direction (`Dir`), automatically looping
    /// back to the beginning if reaching the last client and vice versa. Returns the window ID of
    /// the newly focused window.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let mut clients = Clients::default();
    /// let focused_wid = clients.move_focus(Dir::Left);
    /// ```
    pub fn move_focus(&mut self, dir: Dir) -> u32 {
        let (mut idx, default_idx) = match dir {
            Dir::Right => {
                (
                    self.active_client + 1, 
                    0,
                )
            },
            Dir::Left => {
                (
                    self.active_client.checked_sub(1).unwrap_or_else(|| self.clients.len() - 1),
                    self.clients.len() - 1,
                )
            }
        };

        let c = loop {
            let client = match self.clients.get_mut(idx) {
                Some(client) => client,
                None => {
                    // Since default_idx is always either `0` or `clients.len()-1`, it's safe to unwrap
                    // here.
                    idx = default_idx;
                    self.clients.get_mut(idx).unwrap()
                },
            };

            if client.get_type() != &ClientType::Dock {
                break client;
            }

            match dir {
                Dir::Left => idx -= 1,
                Dir::Right => idx += 1, 
            };
        };
        
        // self.set_focused(c);
        c.wid
    }

    /// Verifies if the client with the ID 'wid' is already being managed.
    #[inline]
    pub fn contains(&self, wid: u32) -> bool {
        self.clients.iter().any(|c| c.wid == wid)
    }

    /// Sets the fullscreen state for the clients with wid based on the specified state `state`.
    /// If the wid is equal to 0, sets for the active client.
    pub fn set_fullscreen(&mut self, wid: u32, action: Operation) -> Result<(), String> {
        let client: &mut Client = if wid != 0 {
            match self.clients.iter_mut().find(|c| c.wid == wid) {
                Some(client) => client,
                None => return Err(format!("Client with wid {} not found", wid)),
            }
        } else {
            match self.clients.get_mut(0) {
                Some(client) => client,
                None => return Err("No clients available".to_string()),
            }
        };

        let status = client.set_state(ClientState::Fullscreen, action)?;
        let data = if status { self.conn.WM_STATE_FULLSCREEN() } else { 0 };

        xcb::change_property(
            &self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            client.wid,
            self.conn.WM_STATE(),
            xcb::ATOM_ATOM,
            32,
            &[data],
        );

        self.resize_tiles(util::get_screen(&self.conn));
        Ok(())
    }

    pub fn set_maximized(&mut self, wid: u32, action: Operation) -> Result<(), String> {
        let client: &mut Client = if wid != 0 {
            match self.clients.iter_mut().find(|c| c.wid == wid) {
                Some(client) => client,
                None => return Err(format!("Client with wid {} not found", wid)),
            }
        } else {
            match self.clients.get_mut(0) {
                Some(client) => client,
                None => return Err("No clients available".to_string()),
            }
        };

        _ = client.set_state(ClientState::Maximized, action)?;

        self.resize_tiles(util::get_screen(&self.conn));
        Ok(())
    }
}

impl Clients {
    pub fn resize_tiles(&self, screen: xcb::Screen) {
        let border_size = self.config.border.size;

        // ....
        let screen_w = screen.width_in_pixels() as u32;
        let screen_h = screen.height_in_pixels() as u32;

        let padding_top = self.max_padding_top() as u32;
        let padding_bottom = self.max_padding_bottom() as u32;
        let padding_left = self.max_padding_left() as u32;
        let padding_right = self.max_padding_right() as u32;

        // The available width and height represent the pixels available for drawing windows.
        // They are the total screen dimensions minus the specified paddings.
        let available_w = screen_w - padding_left - padding_right;
        let available_h = screen_h - padding_top - padding_bottom;

        let normal_clients: Vec<&Client> = self.clients
            .iter()
            .filter(|c| c.is_controlled() && c.is_visible())
            .collect();

        // Starting tilling at top-right
        let mut window_x: u32 = self.config.gap_size;
        let mut window_y: u32 = self.config.gap_size + self.max_padding_top() as u32;

        // ...
        let mut window_h: u32 = available_h - (border_size * 2) - (self.config.gap_size * 2);
        let mut window_w: u32 = if normal_clients.len() == 1 { 
            available_w - (border_size * 2) - (self.config.gap_size * 2)
        } else { 
            available_w / 2 - border_size - self.config.gap_size
        };

        for (i, client) in normal_clients.iter().enumerate() {
            if i > 0 {
                // Since the master window always fills the left-middle of the
                // screen, the other windows will only occupy the right-middle portion.
                window_w = (available_w / 2) - (border_size * 2) - (self.config.gap_size * 2);
                // window_w = available_w / 2 - self.config.border - self.config.gap;
                window_x = available_w / 2 + self.config.gap_size;

                // Adjusting the height for each window located in the right-middle portion of the screen
                // to ensure they fit proportionally based on the total number of windows.
                let height_per_window = available_h / (normal_clients.len() - 1) as u32;

                window_y = (height_per_window * (i - 1) as u32) + self.max_padding_top() + self.config.gap_size;
                window_h = if client.wid == normal_clients.last().unwrap().wid {
                    height_per_window - (border_size * 2) - (self.config.gap_size * 2)
                } else {
                    height_per_window - border_size - self.config.gap_size
                };
            }

            xcb::configure_window(
                &self.conn,
                client.wid,
                &[
                    (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, border_size),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, window_h),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, window_w),
                    (xcb::CONFIG_WINDOW_X as u16, window_x),
                    (xcb::CONFIG_WINDOW_Y as u16, window_y),
                ],
            );
        }

        let fullscreen_clients: Vec<&Client> = self.clients
            .iter()
            .filter(|c| c.has_state(&ClientState::Fullscreen) && c.is_controlled() && c.is_visible())
            .collect();

        for client in fullscreen_clients.iter() {
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

        let maximized_clients: Vec<&Client> = self.clients
            .iter()
            .filter(|c| c.has_state(&ClientState::Maximized) && c.is_controlled() && c.is_visible())
            .collect();

        for client in maximized_clients.iter() {
            xcb::configure_window(
                &self.conn,
                client.wid,
                &[
                    (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, 0),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, available_h),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, available_w),
                    (xcb::CONFIG_WINDOW_X as u16, 0 + padding_left - padding_right),
                    (xcb::CONFIG_WINDOW_Y as u16, 0 + padding_top - padding_bottom),
                ],
            );
        }

        self.conn.flush();
    }

    /// Refreshes the "_NET_CLIENT_LIST" with the current list of clients.
    #[inline]
    pub(self) fn refresh_client_list(&self) {
        ewmh::set_client_list(
            &self.conn,
            0,
            &self.clients.iter().map(|c| c.wid).collect::<Vec<u32>>(),
        );
    }

    /// Returns the maximum padding at the top among all clients.
    ///
    /// If there are no clients or if all clients have no padding at the top,
    /// the function returns 0.
    #[inline]
    pub(self) fn max_padding_top(&self) -> u32 {
        self.clients.iter().map(|c| c.padding_top).max().unwrap_or(0)
    }

    /// Returns the maximum padding at the bottom among all clients.
    ///
    /// If there are no clients or if all clients have no padding at the bottom,
    /// the function returns 0.
    #[inline]
    pub(self) fn max_padding_bottom(&self) ->  u32{
        self.clients.iter().map(|c| c.padding_bottom).max().unwrap_or(0)
    }

    /// Returns the maximum padding at the left among all clients.
    ///
    /// If there are no clients or if all clients have no padding at the left,
    /// the function returns 0.
    #[inline]
    pub(self) fn max_padding_left(&self) -> u32 {
        self.clients.iter().map(|c| c.padding_left).max().unwrap_or(0)
    }

    /// Returns the maximum padding at the right among all clients.
    ///
    /// If there are no clients or if all clients have no padding at the right,
    /// the function returns 0.
    #[inline]
    pub(self) fn max_padding_right(&self) -> u32 {
        self.clients.iter().map(|c| c.padding_right).max().unwrap_or(0)
    }
}
