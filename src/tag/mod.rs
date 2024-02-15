mod geometry;

use std::{sync::Arc, collections::VecDeque};

use xcb_util::ewmh;

use crate::{
    client::{
        Client,
        ClientState,
        ClientID,
    },
    errors::Error,
    layout::Layout, config::Config, util::{self, math},
};

pub use crate::tag::geometry::TagGeometry;

pub type TagID = u32;

#[derive(Clone)]
pub struct Tag {
    /// EWMH | XCB connection.
    conn: Arc<ewmh::Connection>,

    /// Specifies the tag that XCB must use in certain operations. It starts at 0 and is
    /// automatically incremented.
    pub id: TagID,
    
    /// The name of the tag, used to define "_NET_DESKTOP_NAMES".
    pub alias: String,

    pub geo: TagGeometry,

    /// ID of the currently focused client. It is 0 when no client is focused.
    focused_cid: ClientID,

    clients: VecDeque<Client>,
}

impl Tag {
    pub fn new(conn: Arc<ewmh::Connection>, id: u32, alias: &str, width: u32, height: u32) -> Self {
        // TODO: better message
        log::trace!(
            "creating tag. id={} alias={} geo={{width={} height={}}}",
            id,
            alias,
            width,
            height,
        );

        Self {
            id,
            conn,
            alias: alias.to_owned(),
            focused_cid: 0,
            clients: VecDeque::new(),
            geo: TagGeometry {
                w: width, 
                avail_w: width,
                h: height, 
                avail_h: height,
                paddings: [0, 0, 0, 0],
            },
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

    fn set_paddings(&mut self, top: u32, bottom: u32, left: u32, right: u32) {
        self.geo.paddings[0] = self.geo.paddings[0].max(top);
        self.geo.paddings[1] = self.geo.paddings[1].max(bottom);
        self.geo.paddings[2] = self.geo.paddings[2].max(left);
        self.geo.paddings[3] = self.geo.paddings[3].max(right);

        // TODO: remove this!
        if self.alias != "sticky_clients" {
            self.geo.avail_w =  self.geo.w - left - right;
            self.geo.avail_h = self.geo.h - top - bottom;
        }
    }

    /// Manages a new client by adding it to the front of the client list. 
    /// Note: It does not update the "_NET_CLIENT_LIST"; use `Screen::refresh()` for that purpose.
    pub fn manage_client(&mut self, client: Client) {
        self.set_paddings(
            client.geo.paddings[0],
            client.geo.paddings[1],
            client.geo.paddings[2],
            client.geo.paddings[3],
        );

        self.clients.push_front(client);
    }

    /// Removes a client with the specified window ID from the client list.
    /// Note: It does not update the "_NET_CLIENT_LIST"; use `Screen::refresh()` for that purpose.
    pub fn unmanage_client(&mut self, wid: ClientID) {
        self.clients.retain(|c| c.id != wid);

        self.set_paddings(
            self.clients.iter().map(|c| c.geo.paddings[0]).max().unwrap_or(0),
            self.clients.iter().map(|c| c.geo.paddings[1]).max().unwrap_or(0),
            self.clients.iter().map(|c| c.geo.paddings[2]).max().unwrap_or(0),
            self.clients.iter().map(|c| c.geo.paddings[3]).max().unwrap_or(0),
        );
    }

    /// Retrieves an immutable reference to the fisrt client that matches with predicate.
    pub fn get_first_client_when(&self, predicate: impl Fn(&Client) -> bool) -> Result<&Client, Error> {
        self.clients
            .iter()
            .find(|c| predicate(c))
            .ok_or(Error::ClientNotFound(0))
    }

    /// Retrieves an immutable reference to the client with the specified ID.
    // pub fn get_client(&self, id: ClientID) -> Result<&Client, Error> {
    //     self.clients
    //         .iter()
    //         .find(|c| c.id == id)
    //         .ok_or(Error::ClientNotFound(id))
    // }
    
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

    /// Retrieves an immutable reference to a client by its relative index to another client's ID.
    /// If `relative` is `None`, the focused client will be used as a reference.
    pub fn get_client_byidx(&self, i: i32, relative: Option<ClientID>) -> Option<&Client> {
        // Returns the focused client if idx == 0.
        if i == 0 {
            return self.get_focused_client().ok();
        }

        // Iterate only over visible clients.
        let clients: Vec<&Client> = self.clients
            .iter()
            .filter(|c| c.get_state() != ClientState::Hidden)
            .collect();

        match clients.len() {
            0 => return None,
            1 => return Some(&self.clients[0]),
            _ => (),
        };

        // Uses the focused client as a reference if `relative` is `None`.
        let relative = relative.unwrap_or(self.focused_cid);
        let relative_idx = clients.iter().position(|c| c.id == relative)? as i32;

        let target = math::cycle_idx(clients.len(), relative_idx + i)?;
        self.clients.get(target)
    }


    /// Sets focus on a client with the specified window ID, updating the border to `active_color`
    /// and setting the client as the input focus. If there's another focused client, updates its
    /// border to `inactive_color`.
    ///
    /// Returns `Some(true)` if the focus is set successfully, otherwise `None`.
    pub fn focus_client(&mut self, wid: ClientID) -> Option<bool> {
        self.focus_client_if(wid, |_| true)
    }

    /// Same as `Tag::set_focused_client()` but only set the client as focused if the predicate
    /// evaluates `true`.
    pub fn focus_client_if<P>(&mut self, wid: ClientID, predicate: P) -> Option<bool>
    where
        P: Fn(&Client) -> bool
    {
        if let Some(c) = self.clients.iter().find(|c| c.id == wid) {
            if !predicate(c) {
                return Some(false)
            }

            let config = Config::current();

            // Sets the border of the previously focused client to an inactive state, if applicable.
            self.clients
                .iter()
                .find(|c| c.id == self.focused_cid)
                .map(|c| c.set_border(&self.conn, config.border.color_normal));
            
            self.focused_cid = c.id;
            c.set_input_focus(&self.conn); // TODO: make this a tag method
            c.set_border(&self.conn, config.border.color_active);

            return Some(true)
        }

        None
    }

    /// Sets focus on a client by its relative index to another client's ID. updating the border to
    /// `active_color` and setting the client as the input focus. If there's another focused
    /// client, update border to `inactive_color`.
    ///
    /// If `relative` is `None`, the focused client will be used as a reference.
    ///
    /// Returns `Ok(())` if the focus is set successfully, otherwise `Err(Error::ClientNotFound())`.
    pub fn focus_client_byidx(&mut self, idx: i32, relative: Option<ClientID>) -> Result<(), Error> {
        let client = self.get_client_byidx(idx, relative).ok_or(Error::ClientNotFound(0))?;
        self.focus_client(client.id);

        Ok(())
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

    /// Changes the position of the client with window ID `wid_i` with the client with window ID `wid_j`.
    /// Returns `None` if either client does not exist.
    pub fn swap(&mut self, wid_i: ClientID, wid_j: ClientID) -> Option<()> {
        match (self.get_client_idx(wid_i), self.get_client_idx(wid_j)) {
            (Some(i), Some(j)) => Some(self.clients.swap(i, j)),
            _ => None,
        }
    }

    pub fn arrange<T>(&mut self, layout: &T, sticky: &Tag)
    where
        T: Layout
    {
        // Create a new geometry to rearrange from. This geometry must be the merge result of the self
        // and the sticky tag.
        let geometry = TagGeometry::new(
            self.geo.w,
            self.geo.h,
            [
                self.geo.padding_top().max(sticky.geo.padding_top()),
                self.geo.padding_bottom().max(sticky.geo.padding_bottom()),
                self.geo.padding_left().max(sticky.geo.padding_left()),
                self.geo.padding_right().max(sticky.geo.padding_right()),
            ],
        );

        // Maximized and fullscreen clients will not be passed to the layout arrange.
        self.clients
            .iter_mut()
            .filter(|c| {
                (c.get_state() == ClientState::Maximized || c.get_state() == ClientState::Fullscreen) && c.is_controlled()
            })
            .for_each(|c| {
                if c.get_state() == ClientState::Maximized {
                    c.geo.border = 0;
                    c.geo.w = geometry.avail_w;
                    c.geo.h = geometry.avail_h;
                    c.geo.x = geometry.padding_left();
                    c.geo.y = geometry.padding_top();
                } else {
                    c.geo.border = 0;
                    c.geo.w = geometry.w;
                    c.geo.h = geometry.h;
                    c.geo.x = 0;
                    c.geo.y = 0;
                }
            });

        // Only "Tile" clients needs to be passed to the layout arrange.
        let tiled_clients = &mut self.clients
            .iter_mut()
            .filter(|c| c.get_state() == ClientState::Tile && c.is_controlled())
            .collect::<Vec<&mut Client>>();

        let config = Config::current();

        // REMOVE
        tiled_clients.iter_mut().for_each(|c| c.geo.border = config.border.width);

        if tiled_clients.len() == 1 {
            let c = tiled_clients.get_mut(0).unwrap();

            c.geo.x = config.useless_gap + geometry.padding_left(); 
            c.geo.w = geometry.avail_w - (c.geo.border * 2) - (config.useless_gap * 2);

            c.geo.y = config.useless_gap + geometry.padding_top();
            c.geo.h = geometry.avail_h - (c.geo.border * 2) - (config.useless_gap * 2);

            c.geo.x = c.geo.x.max(1);
            c.geo.y = c.geo.y.max(1);
        } else if tiled_clients.len() > 1 {
            layout.arrange(geometry, config.useless_gap, tiled_clients);
        }

        self.clients
            .iter()
            .for_each(|c| {
                xcb::configure_window(
                    &self.conn,
                    c.id,
                    &[
                        (xcb::CONFIG_WINDOW_WIDTH as u16, c.geo.w),
                        (xcb::CONFIG_WINDOW_HEIGHT as u16, c.geo.h),
                        (xcb::CONFIG_WINDOW_X as u16, c.geo.x),
                        (xcb::CONFIG_WINDOW_Y as u16, c.geo.y),
                        (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, c.geo.border),
                    ],
                );
            });
    }
}
