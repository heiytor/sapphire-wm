use std::{sync::Arc, collections::VecDeque};

use xcb_util::ewmh;

use crate::util;

pub struct Client {
    wid: u32,
    is_controlled: bool,
    is_fullscreen: bool,
    is_dock: bool,

    pub padding_top: u32,
    pub padding_bottom: u32,
    pub padding_left: u32,
    pub padding_right: u32,
}

impl Client {
    pub fn new(wid: u32) -> Self {
        Client { 
            wid,
            is_controlled: true,
            is_fullscreen: false,
            is_dock: false,
            padding_top: 0,
            padding_bottom: 0,
            padding_left: 0,
            padding_right: 0,
        }
    }

    pub fn set_docker(&mut self) {
        self.is_dock = true;
    }

    pub fn remove_controll(&mut self) {
        self.is_controlled = false;
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
        self.resize_tiles(util::get_screen(&self.conn));
    }

    /// Unmanages an X Window. Removes it from the "_NET_CLIENT_LIST", and if the window
    /// is the "_NET_ACTIVE_WINDOW", sets the last created window as active.
    pub fn unmanage(&mut self, wid: u32) {
        self.clients.retain(|c| c.wid != wid);

        let last_wid: Option<u32> = self.clients.front().map(|c| c.wid);
        let last_wid: u32 = last_wid.unwrap_or(xcb::WINDOW_NONE);

        self.set_active(last_wid);
        self.refresh_client_list();
        self.resize_tiles(util::get_screen(&self.conn));
    }

    pub fn resize_tiles(&self, screen: xcb::Screen) {
        println!("resize_tiles");
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
            .filter(|c| c.is_controlled && !c.is_fullscreen)
            .collect();

        // Starting tilling at top-right
        let mut window_x: u32 = 0;
        let mut window_y: u32 = 0 + self.max_padding_top() as u32;

        // ...
        let mut window_h: u32 = available_h - self.config.border * 2;
        let mut window_w: u32 = if normal_clients.len() == 1 { 
            available_w - self.config.border * 2
        } else { 
            available_w / 2 - self.config.border
        };

        for (i, client) in normal_clients.iter().enumerate() {
            if i > 0 {
                // Since the master window always fills the left-middle of the
                // screen, the other windows will only occupy the right-middle portion.
                window_w = (available_w / 2) - self.config.border * 2;
                window_x = available_w / 2;

                // Adjusting the height for each window located in the right-middle portion of the screen
                // to ensure they fit proportionally based on the total number of windows.
                let height_per_window = available_h / (normal_clients.len() - 1) as u32;

                window_y = (height_per_window * (i - 1) as u32) + self.max_padding_top() as u32;
                window_h = if client.wid == normal_clients.last().unwrap().wid {
                    height_per_window - (self.config.border * 2)
                } else {
                    height_per_window - self.config.border
                };
            }

            println!("resize window: {}", client.wid);
            println!("{} {}", window_x, window_y);
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

        let fullscreen_clients: Vec<&Client> = self.clients
            .iter()
            .filter(|c| c.is_fullscreen)
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
}

pub enum Dir {
    Left,
    Right,
}

pub enum ClientState {
    Add,
    Remove,
    Toggle,
    Unknown,
}

pub const MASTER_CLIENT: usize = 0; 

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

        // TODO: Ignore docks when moving focus. Perhaps implement it like this:
        // let target_client = loop {
        //     if let Some(client) = self.clients.get(idx) {
        //         if !client.is_dock {
        //             break client;
        //         }
        //         match dir {
        //             Dir::Left => idx -= 1,
        //             Dir::Right => idx += 1,
        //         }
        //     } else {
        //         break &self.clients[default_idx]; // bug when default_idx is a dock.
        //     }
        // };
        
        let wid: u32 = self.clients.get(idx).map_or_else(
            || self.clients[default_idx].wid,
            |client| client.wid,
        );

        self.set_active(wid);
    }

    /// Verifies if the client with the ID 'wid' is already being managed.
    #[inline]
    pub fn contains(&self, wid: u32) -> bool {
        self.clients.iter().any(|c| c.wid == wid)
    }

    /// Sets the fullscreen state for the clients with wid based on the specified state `state`.
    /// If the wid is equal to 0, sets for the active client.
    pub fn set_fullscreen(&mut self, wid: u32, state: ClientState) -> Result<(), String> {
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

        let status = match state {
            ClientState::Add => true,
            ClientState::Remove => false,
            ClientState::Toggle => !client.is_fullscreen,
            ClientState::Unknown => return Err("Invalid state.".to_string()),
        };

        if status == client.is_fullscreen {
            return Ok(())
        }

        let data = if status { self.conn.WM_STATE_FULLSCREEN() } else { 0 };
        client.is_fullscreen = status;

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
