use core::fmt;

use xcb_util::ewmh;

use crate::client::{Client, ClientID};

#[derive(Clone, PartialEq, Debug)]
pub enum ClientType {
    Normal,
    Dock,
    Dialog,
    Splash,
}

impl fmt::Display for ClientType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::Dock => write!(f, "Dock"),
            Self::Dialog => write!(f, "Dialog"),
            Self::Splash => write!(f, "Splash"),
        }
    }
}

impl ClientType {
    /// Retrieves the client's type with the id `id`. The most preferable type is the first and
    /// must include at least one type.
    #[must_use]
    pub fn from_atoms(conn: &ewmh::Connection, id: ClientID) -> Vec<ClientType> {
        let atoms = ewmh::get_wm_window_type(conn, id)
            .get_reply()
            .map_or(vec![], |t| t.atoms().to_owned());

        atoms
            .iter()
            .filter_map(
                |&atom| {
                    match atom {
                        t if t == conn.WM_WINDOW_TYPE_DIALOG() => Some(Self::Dialog),
                        t if t == conn.WM_WINDOW_TYPE_DOCK() => Some(Self::Dock),
                        t if t == conn.WM_WINDOW_TYPE_SPLASH() => Some(Self::Splash),
                        t if t == conn.WM_WINDOW_TYPE_NORMAL() => Some(Self::Normal),
                        _ => None,
                    }
                },
            )
            .collect()
    }
}

impl Client {
    /// Retrieves the client's most preferable type.
    pub fn preferable_type(&self) -> Option<ClientType> {
        self.types.get(0).cloned()
    }
}

