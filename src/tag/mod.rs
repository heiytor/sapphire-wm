use std::{sync::Arc, collections::VecDeque};

use xcb_util::ewmh;

use crate::{
    util,
    config::Config,
    clients::{
        Client,
        WindowID,
        client_state::ClientState,
    },
};

pub enum Dir {
    Left,
    Right,
}

#[derive(Default)]
pub struct Tag {
    id: u32,
    
    // TODO: remove this allow
    #[allow(dead_code)]
    alias: String,

    /// 0 when no client is focused
    focused_wid: WindowID,

    pub clients: VecDeque<Client>,
}

impl Tag {
    pub fn new(id: u32, alias: &str) -> Self {
        Self {
            id,
            alias: alias.to_owned(),
            focused_wid: 0,
            clients: VecDeque::new(),
        }
    }
}

impl Tag {
    /// Manages a new client by adding it to the front of the client list. 
    /// Note: It does not update the "_NET_CLIENT_LIST"; use `Manager::refresh()` for that purpose.
    pub fn manage(&mut self, client: Client) {
        self.clients.push_front(client);
    }

    /// Removes a client with the specified window ID from the client list.
    /// Note: It does not update the "_NET_CLIENT_LIST"; use `Manager::refresh()` for that purpose.
    pub fn unmanage(&mut self, wid: WindowID) {
        self.clients.retain(|c| c.wid != wid);
    }

    fn get_idx(&self, wid: WindowID) -> Option<usize> {
        self.clients.iter().position(|c| c.wid == wid)
    }

    /// Retrieves an immutable reference to the fisrt client that matches with predicate.
    pub fn get_first_when(&self, predicate: impl Fn(&Client) -> bool) -> Option<&Client> {
        self.clients.iter().find(|c| predicate(c))
    }

    /// Returns an immutable reference to the client list.
    pub fn get_all(&self) -> &VecDeque<Client> {
        &self.clients
    }

    /// Retrieves an immutable reference to the client with the specified window ID.
    pub fn get(&self, wid: WindowID) -> Option<&Client> {
        self.clients.iter().find(|c| c.wid == wid)
    }

    /// Retrieves an immutable reference to the focused client.
    pub fn get_focused(&self) -> Option<&Client> {
        self.clients.iter().find(|c| c.wid == self.focused_wid)
    }

    /// Retrieves a mutable reference to the focused client.
    pub fn get_focused_mut(&mut self) -> Option<&mut Client> {
        self.clients.iter_mut().find(|c| c.wid == self.focused_wid)
    }

    /// Sets focus on a client with the specified window ID, updating the border to `active_color`
    /// and setting the client as the input focus. If there's another focused client, updates its
    /// border to `inactive_color`.
    ///
    /// Returns `Some(true)` if the focus is set successfully, otherwise `None`.
    pub fn set_focused(&mut self, conn: &ewmh::Connection, wid: WindowID) -> Option<bool> {
        if let Some(c) = self.clients.iter().find(|c| c.wid == wid) {
            // Sets the border of the previously focused client to an inactive state, if applicable.
            self.clients
                .iter()
                .find(|c| c.wid == self.focused_wid)
                .map(|c| c.set_border(conn, 0xFFF200));

            self.focused_wid = c.wid;
            c.set_input_focus(conn);
            c.set_border(conn, 0xC800FF);

            Some(true)
        } else {
            None
        }
    }

    /// Sets focus on a client with the specified window ID, updating the border to `active_color`
    /// and setting the client as the input focus. If there's another focused client, updates its
    /// border to `inactive_color`.
    ///
    /// Returns `Some(true)` if the focus is set successfully, `Some(false)` if the predicate is not satisfied,
    /// otherwise `None`.
    pub fn set_focused_if(&mut self, conn: &ewmh::Connection, wid: WindowID, predicate: impl Fn(&Client) -> bool) -> Option<bool> {
        if let Some(c) = self.clients.iter().find(|c| c.wid == wid) {
            if !predicate(c) {
                return Some(false)
            }

            // Sets the border of the previously focused client to an inactive state, if applicable.
            self.clients
                .iter()
                .find(|c| c.wid == self.focused_wid)
                .map(|c| c.set_border(conn, 0xFFF200));

            self.focused_wid = c.wid;
            c.set_input_focus(conn);
            c.set_border(conn, 0xC800FF);

            return Some(true)
        }

        None
    }

    /// Returns a tuple representing the maximum padding values (top, bottom, left, right)
    /// among all clients in the client list.
    #[inline]
    pub fn paddings(&self) -> (u32, u32, u32, u32) {
        let top = self.clients.iter().map(|c| c.padding_top).max().unwrap_or(0);
        let bottom = self.clients.iter().map(|c| c.padding_bottom).max().unwrap_or(0);
        let left = self.clients.iter().map(|c| c.padding_left).max().unwrap_or(0);
        let right = self.clients.iter().map(|c| c.padding_right).max().unwrap_or(0);

        (top, bottom, left, right)
    }

    /// Walks `n` clients in the specified direction `dir`, targeting the first client
    /// that matches with the `predicate`. It automatically loops through the clients
    /// vector.
    ///
    /// Returns the window ID of the target client, or `None` in two situations:
    /// 1. When `clients.len()` is equal to 0.
    /// 2. When the function has looped through the clients vector but no matching client is found.
    ///
    /// !Note: Currently, the function only supports `n == 1`.
    pub fn walk(&self, n: usize, dir: Dir, predicate: impl Fn(&Client) -> bool) -> Option<WindowID> {
        if self.clients.len() == 0 {
            return None;
        }

        let find_idx = |std_idx| -> usize {
            self.clients
                .iter()
                .position(|c| c.wid == self.focused_wid)
                .unwrap_or(std_idx)
        };

        let (mut idx, std_idx) = match dir {
            Dir::Right => {
                let std_idx = 0;
                (find_idx(std_idx) + n, std_idx)
            },
            Dir::Left => {
                let std_idx = self.clients.len().checked_sub(n).unwrap_or(0);
                (find_idx(std_idx).checked_sub(n).unwrap_or(std_idx), std_idx)
            },
        };

        // Store the original target index to detect when the vector has looped.
        let already_looped = idx;
        loop {
            let client = self.clients.get(idx).unwrap_or_else(|| {
                // If no client is found with the current index, reset `idx` to `std_idx`, effectively
                // looping through the vector.
                idx = std_idx;
                // Since std_idx is always either `0` or `clients.len()-1`, it's safe to unwrap here.
                self.clients.get(idx).unwrap()
            });

            // TODO:
            if predicate(client) {
                return Some(client.wid)
            }

            idx = match dir {
                Dir::Left => idx.checked_sub(n).unwrap_or(0),
                Dir::Right => idx + n,
            };

            if idx == already_looped {
                return None;
            }
        };
    }

    /// Changes the position of the client with window ID `wid_i` with the client with window ID `wid_j`.
    /// Returns `None` if either client does not exist.
    pub fn swap(&mut self, wid_i: WindowID, wid_j: WindowID) -> Option<()> {
        match (self.get_idx(wid_i), self.get_idx(wid_j)) {
            (Some(i), Some(j)) => Some(self.clients.swap(i, j)),
            _ => None,
        }
    }
}

pub struct Manager {
    conn: Arc<ewmh::Connection>,
    tags: Vec<Tag>,
    config: Arc<Config>,
}

impl Manager {
    pub fn new(conn: Arc<ewmh::Connection>, tags: Vec<Tag>, config: Arc<Config>) -> Self {
        Self {
            conn,
            tags,
            config,
        }
    }
}

#[allow(dead_code)]
impl Manager {
    pub fn get_tag(&self, tag: u32) -> Option<&Tag> {
        self.tags.iter().find(|t| t.id == tag)
    }

    pub fn get_tag_mut(&mut self, tag: u32) -> Option<&mut Tag> {
        self.tags.iter_mut().find(|t| t.id == tag)
    }
    
    pub fn update_tag(&self, t: u32) {
        let tag = self.tags.get(t as usize).unwrap();
        let clients = tag.get_all();

        let border_size = self.config.border.size;

        // ....
        let screen = util::get_screen(&self.conn);
        let screen_w = screen.width_in_pixels() as u32;
        let screen_h = screen.height_in_pixels() as u32;

        let (padding_top, padding_bottom, padding_left, padding_right) = tag.paddings();

        // The available width and height represent the pixels available for drawing windows.
        // They are the total screen dimensions minus the specified paddings.
        let available_w = screen_w - padding_left - padding_right;
        let available_h = screen_h - padding_top - padding_bottom;

        let normal_clients: Vec<&Client> = clients
            .iter()
            .filter(|c| c.is_controlled() && c.is_visible())
            .collect();

        // Starting tilling at top-right
        let mut window_x: u32 = self.config.gap_size;
        let mut window_y: u32 = self.config.gap_size + padding_top;

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

                window_y = (height_per_window * (i - 1) as u32) + padding_top + self.config.gap_size;
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

        let maximized_clients: Vec<&Client> = clients
            .iter()
            .filter(|c| c.last_state() == &ClientState::Maximized && c.is_controlled() && c.is_visible())
            .collect();

        for client in maximized_clients.iter() {
            xcb::configure_window(
                &self.conn,
                client.wid,
                &[
                    (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, 0),
                    (xcb::CONFIG_WINDOW_HEIGHT as u16, available_h),
                    (xcb::CONFIG_WINDOW_WIDTH as u16, available_w),
                    (xcb::CONFIG_WINDOW_X as u16, 0 + padding_left),
                    (xcb::CONFIG_WINDOW_Y as u16, 0 + padding_top),
                ],
            );
        }

        let fullscreen_clients: Vec<&Client> = clients
            .iter()
            .filter(|c| c.last_state() == &ClientState::Fullscreen && c.is_controlled() && c.is_visible())
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


        self.conn.flush();
    }

    /// Refreshes the "_NET_CLIENT_LIST" with the current list of clients in all tags.
    pub fn refresh(&self) {
        // TODO: make it less verbose and more performatic
        let mut clients: VecDeque<&Client> = VecDeque::new();
        for t in self.tags.iter() {
            for c in t.get_all() {
                clients.push_front(c)
            }
        }

        ewmh::set_client_list(
            &self.conn,
            0,
            &clients.iter().map(|c| c.wid).collect::<Vec<u32>>(),
        );
    }

}
