mod action;
mod kind;
mod geometry;
mod state;

use xcb_util::{ewmh, icccm};

use crate::util as gutil; // TODO: change this!!!!!!

pub use crate::client::{
    action::ClientAction,
    kind::ClientType,
    geometry::ClientGeometry,
    state::ClientState,
};

/// Represents the ID of the client. Typically the `event.window()`, `event.child()` or
/// `event.event()` in XCB events.
pub type ClientID = u32;

#[derive(Clone)]
pub struct Client {
    /// Represents the ID of the client. Typically the `event.window()`, `event.child()` or
    /// `event.event()` in XCB events.
    pub id: ClientID,

    /// The `_NET_WM_PID` of the client, also known as the process ID.
    pub wm_pid: Option<u32>,

    /// The `WM_CLASS` of the client.
    pub wm_class: Option<String>,

    /// The `WM_NAME` of the client.
    pub wm_name: Option<String>,

    pub geo: ClientGeometry,

    is_controlled: bool,

    /// Represents the list of types associated with a client. Each type must be unique in the vector.
    /// Typically, a client has a single unique type. However, in cases where a client has multiple types,
    /// the first one is considered the most preferable.
    ///
    /// Refer to: https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm45912237346656
    types: Vec<ClientType>,
    
    /// Represents the list of current `xcb::WM_STATE` atoms of the client.
    /// Each state must be unique in the vector.
    ///
    /// The importance of states is from last to first, as the latest pushed states
    /// are treated with more privileges. For example, a client with `states` equal to
    /// `[ClientState::Fullscreen, ClientState::Maximized]` must be drawn as maximized.
    /// When removing `ClientState::Maximized` from the list, the client must be drawn as fullscreen.
    ///
    /// Some functions that returns the state may sometimes return `ClientState::Tile`. This state
    /// is special and is never included in the list; it simply indicates that the client
    /// doesn't have any configured state.
    ///
    /// Refer to: https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm46201142858672
    states: Vec<ClientState>,

    /// Represents the list of current `_NET_ALLOWED_ACTIONS` atoms of the client.
    /// Each action must be unique in the vector.
    ///
    /// Refer to: https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm46201142837824
    allowed_actions: Vec<ClientAction>,

    protocols: Vec<u32>,
}

impl Client {
    pub fn new(conn: &ewmh::Connection, id: ClientID) -> Self {
        let mut client = Self {
            id,
            is_controlled: false,
            states: vec![ClientState::Tile],
            allowed_actions: vec![],
            types: vec![],
            protocols: vec![],
            wm_class: None,
            wm_pid: None,
            wm_name: None,
            geo: ClientGeometry {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
                border: 0,
                paddings: [0, 0, 0, 0],
            },
        };

        if let Ok(r) = icccm::get_wm_class(conn, id).get_reply() {
            client.wm_class = Some(r.class().to_owned());
        }

        if let Ok(r) = icccm::get_wm_name(conn, id).get_reply() {
            client.wm_name = Some(r.name().to_owned());
        }

        if let Ok(p) = ewmh::get_wm_pid(conn, id).get_reply() {
            client.wm_pid = Some(p);
        }

        if let Ok(s) = ewmh::get_wm_strut_partial(conn, id).get_reply() {
            client.geo.paddings[0] = s.top;
            client.geo.paddings[1] = s.bottom;
            client.geo.paddings[2] = s.left;
            client.geo.paddings[3] = s.right;
        };

        // TODO: maybe a custom enum with the supported protocols?
        client.protocols = xcb_util::icccm::get_wm_protocols(conn, id, conn.WM_PROTOCOLS())
            .get_reply()
            .map_or(
                vec![],
                |p| p.atoms().to_vec(),
            );

        client.types = ClientType::from_atoms(conn, id);
        client.allow_action(conn, ClientAction::Close);

        if client.preferable_type().is_some_and(|t| t == ClientType::Dock) {
            client.add_state(conn, ClientState::Sticky);
        } else {
            client.is_controlled = true;
            client.allow_actions(
                conn,
                vec![
                    ClientAction::Maximize,
                    ClientAction::Fullscreen,
                    ClientAction::ChangeTag,
                    ClientAction::Resize,
                    ClientAction::Move,
                ],
            );
        }

        client
    }

    /// Maps a window.
    pub fn map(&self, conn: &ewmh::Connection) {
        xcb::map_window(conn, self.id);
    }

    /// Unmaps a window.
    pub fn unmap(&self, conn: &ewmh::Connection) {
        xcb::unmap_window(conn, self.id);
    }

    pub fn set_border(&self, conn: &ewmh::Connection, color: u32) {
        xcb::change_window_attributes(
            conn,
            self.id,
            &[(xcb::CW_BORDER_PIXEL, color)],
        );
    }

    pub fn set_input_focus(&self, conn: &ewmh::Connection) {
        xcb::set_input_focus(
            conn,
            xcb::INPUT_FOCUS_PARENT as u8,
            self.id,
            xcb::CURRENT_TIME
        );
    }

    pub fn has_protocol(&self, atom: xcb::Atom) -> bool {
        self.protocols.contains(&atom)
    }

    pub fn kill(&self, conn: &ewmh::Connection) {
        let wm_delete_window = gutil::get_atom(conn, "WM_DELETE_WINDOW");

        if self.has_protocol(wm_delete_window) {
            let event = xcb::ClientMessageEvent::new(
                32,
                self.id,
                conn.WM_PROTOCOLS(),
                xcb::ClientMessageData::from_data32([
                    wm_delete_window,
                    xcb::CURRENT_TIME,
                    xcb::NONE,
                    xcb::NONE,
                    xcb::NONE,
                ]),
            );

            // TODO: kill with PID when this event fails
            xcb::send_event(
                &conn,
                false,
                self.id,
                xcb::EVENT_MASK_NO_EVENT,
                &event,
            );
        } else {
            xcb::kill_client(conn, self.id);
        }
    }

    #[inline(always)]
    pub fn is_controlled(&self) -> bool {
        self.is_controlled
    }
}
