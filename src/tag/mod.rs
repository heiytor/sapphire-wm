use std::{sync::Arc, collections::VecDeque};

use xcb_util::ewmh;

use crate::{
    client::{
        Client,
        ClientState,
        ClientID,
    },
    errors::Error, layout::Layout,
};

pub enum Dir {
    Left,
    Right,
}

pub type TagID = u32;

pub struct Tag {
    /// EWMH | XCB connection.
    conn: Arc<ewmh::Connection>,

    /// Specifies the tag that XCB must use in certain operations. It starts at 0 and is
    /// automatically incremented.
    pub id: TagID,
    
    /// The name of the tag, used to define "_NET_DESKTOP_NAMES".
    pub alias: String,

    /// ID of the currently focused client. It is 0 when no client is focused.
    focused_cid: ClientID,

    clients: VecDeque<Client>,
}

impl Tag {
    pub fn new(id: u32, alias: &str, conn: Arc<ewmh::Connection>) -> Self {
        Self {
            id,
            conn,
            alias: alias.to_owned(),
            focused_cid: 0,
            clients: VecDeque::new(),
        }
    }

    /// Verifies if the tag contains a client with ID `id`.
    pub fn contains_client(&self, id: ClientID) -> bool {
        self.clients.iter().any(|c| c.id == id)
    }

    /// Get the position of the client with ID `id` in the clients vector. Returns `None` if the
    /// client does not exist.
    fn get_client_idx(&self, id: ClientID) -> Option<usize> {
        self.clients.iter().position(|c| c.id == id)
    }

    /// Manages a new client by adding it to the front of the client list. 
    /// Note: It does not update the "_NET_CLIENT_LIST"; use `Manager::refresh()` for that purpose.
    pub fn manage_client(&mut self, client: Client) {
        self.clients.push_front(client);
    }

    /// Removes a client with the specified window ID from the client list.
    /// Note: It does not update the "_NET_CLIENT_LIST"; use `Manager::refresh()` for that purpose.
    pub fn unmanage_client(&mut self, wid: ClientID) {
        self.clients.retain(|c| c.id != wid);
    }

    /// Retrieves an immutable reference to the fisrt client that matches with predicate.
    pub fn get_first_client_when(&self, predicate: impl Fn(&Client) -> bool) -> Result<&Client, Error> {
        self.clients
            .iter()
            .find(|c| predicate(c))
            .ok_or(Error::ClientNotFound(0))
    }

    /// Retrieves an immutable reference to the client with the specified ID.
    pub fn get_client(&self, id: ClientID) -> Result<&Client, Error> {
        self.clients
            .iter()
            .find(|c| c.id == id)
            .ok_or(Error::ClientNotFound(id))
    }
    
    /// Retrieves an immutable reference to the client with the specified ID.
    pub fn get_client_mut(&mut self, id: ClientID) -> Result<&mut Client, Error> {
        self.clients
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or(Error::ClientNotFound(id))
    }

    /// Retrieves an immutable reference to the focused client.
    pub fn get_focused_client(&self) -> Result<&Client, Error> {
        self.clients
            .iter()
            .find(|c| c.id == self.focused_cid)
            .ok_or(Error::ClientNotFound(self.focused_cid))
    }

    /// Retrieves a mutable reference to the focused client.
    pub fn get_focused_client_mut(&mut self) -> Result<&mut Client, Error> {
        self.clients
            .iter_mut()
            .find(|c| c.id == self.focused_cid)
            .ok_or(Error::ClientNotFound(self.focused_cid))
    }

    /// Sets focus on a client with the specified window ID, updating the border to `active_color`
    /// and setting the client as the input focus. If there's another focused client, updates its
    /// border to `inactive_color`.
    ///
    /// Returns `Some(true)` if the focus is set successfully, otherwise `None`.
    pub fn set_focused_client(&mut self, wid: ClientID) -> Option<bool> {
        self.set_focused_client_if(wid, |_| true)
    }

    /// Same as `Tag::set_focused_client()` but only set the client as focused if the predicate
    /// evaluates `true`.
    pub fn set_focused_client_if(&mut self, wid: ClientID, predicate: impl Fn(&Client) -> bool) -> Option<bool> {
        if let Some(c) = self.clients.iter().find(|c| c.id == wid) {
            if !predicate(c) {
                return Some(false)
            }

            // Sets the border of the previously focused client to an inactive state, if applicable.
            self.clients
                .iter()
                .find(|c| c.id == self.focused_cid)
                .map(|c| c.set_border(&self.conn, 0xFFF200));
            
            self.focused_cid = c.id;
            c.set_input_focus(&self.conn);
            c.set_border(&self.conn, 0xC800FF);

            return Some(true)
        }

        None
    }

    pub fn clone_clients(&self) -> Vec<Client> {
        self.clients.iter().cloned().collect()
    }
    
    /// Maps all visible clients of the tag.
    pub fn map(&self) {
        self.clients
            .iter()
            .for_each(|c| {
                if !c.has_state(&ClientState::Hidden) { c.map(&self.conn) }
            });
    }

    /// Unmaps all visible clients of the tag.
    pub fn unmap(&self) {
        self.clients
            .iter()
            .for_each(|c| {
                if !c.has_state(&ClientState::Hidden) { c.unmap(&self.conn) }
            });
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
    pub fn walk(&self, n: usize, dir: Dir, predicate: impl Fn(&Client) -> bool) -> Option<ClientID> {
        if self.clients.len() == 0 {
            return None;
        }

        let find_idx = |std_idx| -> usize {
            self.clients
                .iter()
                .position(|c| c.id == self.focused_cid)
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
                return Some(client.id)
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
    pub fn swap(&mut self, wid_i: ClientID, wid_j: ClientID) -> Option<()> {
        match (self.get_client_idx(wid_i), self.get_client_idx(wid_j)) {
            (Some(i), Some(j)) => Some(self.clients.swap(i, j)),
            _ => None,
        }
    }
}

pub fn resize_tag<T>(tag: &mut Tag, layout: &T, sticky: &Vec<Client>)
where
    T: Layout
{
    let normal_clients: &mut Vec<&mut Client> = &mut tag.clients
        .iter_mut()
        .filter(|c| c.last_state() != &ClientState::Hidden && c.is_controlled())
        .collect();

    layout.resize_clients(normal_clients);

    tag.clients
        .iter_mut()
        .filter(|c| {
            c.last_state() == &ClientState::Fullscreen && c.is_controlled()
        })
    .for_each(|c| {
        // client.rect.border = 0;
        // client.rect.w = available_w;
        // client.rect.h = available_h;
        // client.rect.x = padding_left;
        // client.rect.y = padding_top;
    });

    tag.clients
        .iter_mut()
        .filter(|c| {
            c.last_state() == &ClientState::Fullscreen && c.is_controlled()
        })
    .for_each(|c| {
        // c.rect.border = 0;
        // c.rect.w = screen_w;
        // c.rect.h = screen_h;
        // c.rect.x = 0;
        // c.rect.y = 0;
    });

    tag.clients.iter().for_each(|c| log::info!("x{} y{}", c.rect.x, c.rect.y))
}
