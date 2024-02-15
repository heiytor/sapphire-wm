use std::sync::Arc;

use xcb_util::ewmh;

use crate::{
    tag::{
        Tag, TagID,
    },
    errors::Error,
    client::Client,
    util,
    layout::LayoutTile,
};

#[derive(Clone)]
pub struct ScreenGeometry {
    pub width: u32,
    pub height: u32,
}

pub struct Screen {
    pub id: i32,

    pub root: u32,

    pub geo: ScreenGeometry,

    /// ID of the currently focused tag. Retrieve the current tag using `Self::get_focused_tag[mut]()`,
    /// as the ID may point to an non existent tag.
    focused_tag_id: TagID,

    /// The ewmh connection.
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
    tags: Vec<Tag>,
}

impl Screen {
    pub fn new(conn: Arc<ewmh::Connection>, id: i32, screen: xcb::Screen) -> Self {
        if let Err(cookie) = xcb::change_window_attributes_checked(
            &conn,
            screen.root(),
            &[(
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY,
            )],
        ).request_check() {
            panic!("Is another window manager running? Error = {}", cookie)
        }

        ewmh::set_supported(
            &conn,
            id,
            &[
                conn.SUPPORTED(),
                conn.SUPPORTING_WM_CHECK(),

                conn.CLIENT_LIST(),

                conn.ACTIVE_WINDOW(),
                conn.CURRENT_DESKTOP(),
                conn.DESKTOP_NAMES(),
                conn.NUMBER_OF_DESKTOPS(),

                conn.WM_STATE(),
                conn.WM_STATE_FULLSCREEN(),
                conn.WM_STATE_MAXIMIZED_VERT(),
                conn.WM_STATE_MAXIMIZED_HORZ(),
                conn.WM_STATE_STICKY(),

                conn.WM_WINDOW_TYPE(),
                conn.WM_WINDOW_TYPE_DOCK(),
                conn.WM_WINDOW_TYPE_NORMAL(),

                conn.WM_ACTION_FULLSCREEN(),
                conn.WM_ACTION_MAXIMIZE_VERT(),
                conn.WM_ACTION_MAXIMIZE_HORZ(),
                conn.WM_ACTION_CLOSE(),
                conn.WM_ACTION_CHANGE_DESKTOP(),
                conn.WM_ACTION_RESIZE(),
                conn.WM_ACTION_MOVE(),
                // conn.WM_ACTION_MINIMIZE(), 

                conn.WM_STRUT(),
                conn.WM_STRUT_PARTIAL(),

                conn.WM_PID(),
            ],
        );

        let tags = vec![
            String::from("1"),
            String::from("2"),
            String::from("3"),
            String::from("4"),
            String::from("5"),
            String::from("6"),
            String::from("7"),
            String::from("8"),
            String::from("9"),
        ];

        // The screen must have at least one tag.
        let mut tags = if !tags.is_empty() {
            tags
                .iter()
                .enumerate()
                .map(|(i, a)| Tag::new(conn.clone(), i as u32, a, screen.width_in_pixels() as u32, screen.height_in_pixels() as u32))
                .collect()
        } else {
            vec![Tag::new(conn.clone(), 0, "1", screen.width_in_pixels() as u32, screen.height_in_pixels() as u32)]
        };
    
        ewmh::set_number_of_desktops(&conn, id, tags.len() as u32);
        ewmh::set_desktop_names(&conn, id, tags.iter().map(|d| d.alias.as_ref()));

        // Create the sticky tag.
        // Since the clients contained in this tag will be shown in all other tags the ID of the
        // tag must be set to the maximum 32-bit number.
        // Reference: https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm46201142872912
        tags.push(Tag::new(conn.clone(), 0xFFFFFFFF, "sticky_clients", 0, 0));

        Self {
            id,
            root: screen.root(),
            conn,
            tags,
            focused_tag_id: 0, // TODO: config.default_focused_tag_id
            geo: ScreenGeometry {
                width: screen.width_in_pixels() as u32,
                height: screen.height_in_pixels() as u32,
            },
        }
    }

    /// Sets the default screen and tag for the window manager.
    pub fn set_defaults(conn: &ewmh::Connection, screen_id: i32, tag_id: u32) {
        ewmh::set_current_desktop(conn, screen_id, tag_id);
    }
}

impl Screen {
    fn set_focused_tag(&mut self, tag_id: TagID) {
        ewmh::set_current_desktop(&self.conn, self.id, tag_id);
        self.focused_tag_id = tag_id;
    }

    pub fn contains_tag(&self, tag_id: TagID) -> bool {
        self.tags.iter().any(|t| t.id == tag_id)
    }

    /// Returns an immutable reference to the sticky tag.
    pub fn sticky_tag(&self) -> &Tag {
        // As the window manager ensures that this tag always exists, it will never be `None`.
        let idx = self.tags.len()-1;
        self.tags.get(idx).unwrap()
    }

    /// Returns a mutable reference to the sticky tag.
    pub fn sticky_tag_mut(&mut self) -> &mut Tag {
        // As the window manager ensures that this tag always exists, it will never be `None`.
        let idx = self.tags.len()-1;
        self.tags.get_mut(idx).unwrap()
    }

    /// Returns a immutable reference to the specified tag or `Error::TagNotFound(id)` when the
    /// provided ID does not exist.
    pub fn get_tag(&self, id: u32) -> Result<&Tag, Error> {
        self.tags
            .iter()
            .find(|t| t.id == id)
            .ok_or(Error::TagNotFound(id))
    }

    /// Returns a mutable reference to the specified tag or `Error::TagNotFound(id)` when the
    /// provided ID does not exist.
    pub fn get_tag_mut(&mut self, id: u32) -> Result<&mut Tag, Error> {
        self.tags
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(Error::TagNotFound(id))
    }

    /// Returns a immutable reference to the focused tag or `Error::TagNotFound(id)` when the
    /// provided ID does not exist.
    pub fn get_focused_tag(&self) -> Result<&Tag, Error> {
        let id = self.focused_tag_id;

        self.tags
            .iter()
            .find(|t| t.id == id)
            .ok_or(Error::TagNotFound(id))
    }

    /// Returns a mutable reference to the focused tag or `Error::TagNotFound(id)` when the
    /// provided ID does not exist.
    pub fn get_focused_tag_mut(&mut self) -> Result<&mut Tag, Error> {
        let id = self.focused_tag_id;

        self.tags
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(Error::TagNotFound(id))
    }

    /// Readjust the layout of the tag with ID `id`. Returns `Error::TagNotFound(id)` when the
    /// provided ID does not exist.
    pub fn arrange_tag(&mut self, id: TagID) -> Result<(), Error> {
        let sticky = self.sticky_tag_mut().clone();

        self.get_tag_mut(id)?
            .arrange(&LayoutTile::new(), &sticky);

        Ok(())
    }

    /// Focuses and view the tag with ID `id`. It will also set the input focus to the focused
    /// client on the tag, if any. Returns `Error::TagNotFound(id)` when the provided ID does not
    /// exist. 
    pub fn view_tag(&mut self, id: u32) -> Result<(), Error> {
        if self.get_focused_tag().is_ok_and(|t| t.id == id) {
            return Ok(())
        }

        let conn = self.conn.clone();

        let tag = self.get_tag_mut(id)?;
        tag.map();
        
        // Set the input focus to the currently focused client on dtag, if one exists; otherwise
        // disable the input.
        match tag.get_focused_client_mut() {
            Ok(c) => c.set_input_focus(&conn),
            Err(_) => util::disable_input_focus(&conn),
        }

        // Before updating the ID of the focused tag, we hide all visible clients on the current
        // focused tag, if any.
        //
        // This follows the second approach outlined in the "Virtual Desktop Implementation note" (https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm45912241712208)
        // of the EWMH.
        if let Ok(tag) = self.get_focused_tag() {
            tag.unmap();
        }

        _ = self.arrange_tag(id);
        self.set_focused_tag(id);

        Ok(())
    }

    /// Moves the currently focused client from the source tag to destination tag. Returns
    /// `Error::TagNotFound(src|dest)` when any provided ID does not exist.
    pub fn move_focused_client(&mut self, src: TagID, dest: TagID) -> Result<(), Error> {
        if !self.contains_tag(src) {
            return Err(Error::TagNotFound(src))
        }

        if !self.contains_tag(dest) {
            return Err(Error::TagNotFound(dest))
        }

        let conn = self.conn.clone();

        // Unmanage and hide the focused client of the source tag.
        let s_tag = self.get_tag_mut(src).unwrap();

        let client = s_tag.get_focused_client_mut().unwrap().clone();
        client.unmap(&conn);
        let client_id = client.id;

        s_tag.unmanage_client(client_id);

        // Set the most recent client as input focus on the source tag if any.
        if let Ok(c) = s_tag.get_first_client_when(|c| c.is_controlled()) {
            s_tag.focus_client(c.id);
        } else {
            util::disable_input_focus(&conn)
        }

        // Move the client to the destination tag
        let d_tag = self.get_tag_mut(dest).unwrap();

        d_tag.manage_client(client);
        d_tag.focus_client(client_id);
        util::set_client_tag(&conn, client_id, dest);

        _ = self.arrange_tag(dest);
        _ = self.arrange_tag(src);

        Ok(())
    }

    /// Refreshes the "_NET_CLIENT_LIST" with the current list of clients in all tags.
    pub fn refresh(&self) {
        // TODO: make it less verbose and more performatic
        let mut clients: Vec<Client> = Vec::new();
        for t in self.tags.iter() {
            let b = t.clone_clients();
            for c in b.into_iter() {
                clients.push(c);
            }
        }

        ewmh::set_client_list(
            &self.conn,
            0,
            &clients.iter().map(|c| c.id).collect::<Vec<u32>>(),
        );
    }
}
