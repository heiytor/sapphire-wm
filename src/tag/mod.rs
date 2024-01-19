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

pub type TagID = u32;

pub enum TagErr {
    NotFound(TagID)
}

impl ToString for TagErr {
    fn to_string(&self) -> String {
        match self {
            TagErr::NotFound(id) => format!["Tag[{}] not found.", id],
        }
    }
}

pub struct Tag {
    id: TagID,
    
    conn: Arc<ewmh::Connection>,

    // TODO: remove this allow
    #[allow(dead_code)]
    alias: String,

    /// 0 when no client is focused
    pub focused_wid: WindowID,

    clients: VecDeque<Client>,
}

impl Tag {
    pub fn new(id: u32, alias: &str, conn: Arc<ewmh::Connection>) -> Self {
        Self {
            id,
            conn,
            alias: alias.to_owned(),
            focused_wid: 0,
            clients: VecDeque::new(),
        }
    }
}

impl Tag {
    pub fn get_id(&self) -> &TagID {
        &self.id
    }

    pub fn contains(&self, wid: WindowID) -> bool {
        self.clients.iter().any(|c| c.wid == wid)
    }

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
    
    /// Returns a mutable reference to the client list.
    pub fn get_all_mut(&mut self) -> &mut VecDeque<Client> {
        &mut self.clients
    }

    /// Retrieves an immutable reference to the client with the specified window ID.
    pub fn get(&self, wid: WindowID) -> Option<&Client> {
        self.clients.iter().find(|c| c.wid == wid)
    }
    
    /// Retrieves an immutable reference to the client with the specified window ID.
    pub fn get_mut(&mut self, wid: WindowID) -> Option<&mut Client> {
        self.clients.iter_mut().find(|c| c.wid == wid)
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
    pub fn set_focused(&mut self, wid: WindowID) -> Option<bool> {
        if let Some(c) = self.clients.iter().find(|c| c.wid == wid) {
            // Sets the border of the previously focused client to an inactive state, if applicable.
            self.clients
                .iter()
                .find(|c| c.wid == self.focused_wid)
                .map(|c| c.set_border(&self.conn, 0xFFF200));

            self.focused_wid = c.wid;
            c.set_input_focus(&self.conn);
            c.set_border(&self.conn, 0xC800FF);

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
    pub fn set_focused_if(&mut self, wid: WindowID, predicate: impl Fn(&Client) -> bool) -> Option<bool> {
        if let Some(c) = self.clients.iter().find(|c| c.wid == wid) {
            if !predicate(c) {
                return Some(false)
            }

            // Sets the border of the previously focused client to an inactive state, if applicable.
            self.clients
                .iter()
                .find(|c| c.wid == self.focused_wid)
                .map(|c| c.set_border(&self.conn, 0xFFF200));

            self.focused_wid = c.wid;
            c.set_input_focus(&self.conn);
            c.set_border(&self.conn, 0xC800FF);

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

    pub fn focus(&mut self) {
        xcb_util::ewmh::set_current_desktop(&self.conn, 0, self.id);
        self.clients.iter_mut().for_each(|c| c.show(&self.conn));

        let conn = self.conn.clone();
        match self.get_mut(self.focused_wid) {
            Some(c) => c.set_input_focus(&conn),
            None => util::disable_input_focus(&self.conn),
        };
    }
}

pub fn redraw(conn: &ewmh::Connection, clients: Vec<Client>, config: &Config) {
    let border_size = config.border.size;

    // ....
    let screen = util::get_screen(conn);
    let screen_w = screen.width_in_pixels() as u32;
    let screen_h = screen.height_in_pixels() as u32;

    let padding_top = clients.iter().map(|c| c.padding_top).max().unwrap_or(0);
    let padding_bottom = clients.iter().map(|c| c.padding_bottom).max().unwrap_or(0);
    let padding_left = clients.iter().map(|c| c.padding_left).max().unwrap_or(0);
    let padding_right = clients.iter().map(|c| c.padding_right).max().unwrap_or(0);

    // The available width and height represent the pixels available for drawing windows.
    // They are the total screen dimensions minus the specified paddings.
    let available_w = screen_w - padding_left - padding_right;
    let available_h = screen_h - padding_top - padding_bottom;

    let normal_clients: Vec<&Client> = clients
        .iter()
        .filter(|c| c.is_controlled() && c.is_visible())
        .collect();

    // Starting tilling at top-right
    let mut window_x: u32 = config.gap_size;
    let mut window_y: u32 = config.gap_size + padding_top;

    // ...
    let mut window_h: u32 = available_h - (border_size * 2) - (config.gap_size * 2);
    let mut window_w: u32 = if normal_clients.len() == 1 { 
        available_w - (border_size * 2) - (config.gap_size * 2)
    } else { 
        available_w / 2 - border_size - config.gap_size
    };

    for (i, client) in normal_clients.iter().enumerate() {
        if i > 0 {
            // Since the master window always fills the left-middle of the
            // screen, the other windows will only occupy the right-middle portion.
            window_w = (available_w / 2) - (border_size * 2) - (config.gap_size * 2);
            // window_w = available_w / 2 - self.config.border - self.config.gap;
            window_x = available_w / 2 + config.gap_size;

            // Adjusting the height for each window located in the right-middle portion of the screen
            // to ensure they fit proportionally based on the total number of windows.
            let height_per_window = available_h / (normal_clients.len() - 1) as u32;

            window_y = (height_per_window * (i - 1) as u32) + padding_top + config.gap_size;
            window_h = if client.wid == normal_clients.last().unwrap().wid {
                height_per_window - (border_size * 2) - (config.gap_size * 2)
            } else {
                height_per_window - border_size - config.gap_size
            };
        }

        xcb::configure_window(
            &conn,
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
            &conn,
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
            &conn,
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
}

pub struct Manager {
    conn: Arc<ewmh::Connection>,

    /// Stores information about all the tags in the window manager. Each tag is responsible for
    /// managing its own clients. This vector is never empty, and must store at least 1 valid tag.
    ///
    /// The last tag in the vector is reserved for storing "sticky" clients. `Sticky` clients are
    /// those that the window manager must keep on the screen even when changing tags, such as
    /// docks, SapphireWM always ensures that this tag exists and does not allow "normal" clients
    /// (e.g., terminals) to be in sticky mode.
    /// See: https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm46201142867040
    /// Use either `Manager::sticky_tag()` or `Manager::sticky_tag_mut()` to retrieve such
    /// clients.
    // TODO: 
    // Maybe this approach is too expensive and we should store all clients in a Manager
    // vector.
    tags: Vec<Tag>,

    config: Arc<Config>,

    pub focused_tag_id: u32,
}

impl Manager {
    pub fn new(conn: Arc<ewmh::Connection>, mut tags: Vec<Tag>, config: Arc<Config>) -> Self {
        if tags.is_empty() {
            tags = vec![Tag::new(0, "1", conn.clone())];
        }

        // Create the sticky tag.
        tags.push(Tag::new((tags.len()-1) as u32, "sticky_clients", conn.clone()));

        Self {
            conn,
            tags,
            config,
            focused_tag_id: 0,
        }
    }
}

impl Manager {
    /// Returns an immutable reference to the sticky tag. As the window manager ensures that this
    /// tag always exists, it will never be `None`.
    pub fn sticky_tag(&self) -> &Tag {
        self.tags.get(self.tags.len()-1).unwrap()
    }

    /// Returns a mutable reference to the sticky tag. As the window manager ensures that this tag
    /// always exists, it will never be `None`.
    pub fn sticky_tag_mut(&mut self) -> &mut Tag {
        let idx = self.tags.len()-1;
        self.tags.get_mut(idx).unwrap()
    }

    /// Returns an immutable reference to the specified tag or `None` if it does not exist.
    pub fn get_tag(&self, tag: u32) -> Option<&Tag> {
        self.tags.iter().find(|t| t.id == tag)
    }

    /// Returns a mutable reference to the specified tag or `None` if it does not exist.
    pub fn get_tag_mut(&mut self, tag: u32) -> Option<&mut Tag> {
        self.tags.iter_mut().find(|t| t.id == tag)
    }

    /// Focuses the tag with ID `id`. Returns `TagErr::NotFound(id)` when the provided ID does not
    /// exist. It will remove the previously focused tag, if any; also set the input focus to the
    /// focused client on the newly focused tag, if any.
    pub fn focus_tag(&mut self, id: u32) -> Result<(), String> {
        if self.focused_tag_id == id {
            return Ok(())
        }

        // We save the old focused ID to unfocus the old tag later.
        let old_focused_tag_id = self.focused_tag_id;
        let new_focused_tag = self.get_tag_mut(id).ok_or_else(|| TagErr::NotFound(id).to_string())?;

        new_focused_tag.focus();
        _ = self.draw_clients_from(&[id]);
        self.focused_tag_id = id;

        let conn = self.conn.clone();
        if let Some(t) = self.get_tag_mut(old_focused_tag_id) {
            t.get_all_mut().iter_mut().for_each(|c| c.hide(&conn))
        }

        Ok(())
    }

    /// Draws a list of specified clients on the current tag. The list of clients is built from
    /// the `tags_id`, which fetch the clients from all specified tags and then draw them, effectively
    /// drawing the clients from `n` tags on the current one.
    ///
    /// It also ensures that the `sticky_clients` are always drawn in the window. Refer to `Manager::tags`
    /// for more information about sticky clients.
    pub fn draw_clients_from(&self, tags_id: &[TagID]) -> Result<(), String> {
        // Ensure that the tag is always drawn with sticky clients.
        let mut clients: Vec<Client> = self.sticky_tag().get_all().iter().cloned().collect();

        for id in tags_id {
            let t = self.get_tag(*id).ok_or_else(|| TagErr::NotFound(*id).to_string())?;
            clients.extend(t.get_all().iter().cloned());
        }

        redraw(&self.conn, clients, &self.config);
        Ok(())
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
