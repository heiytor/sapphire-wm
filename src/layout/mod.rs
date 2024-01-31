mod tile;

use crate::client::Client;

pub use crate::layout::tile::LayoutTile;

pub trait Layout {
    /// TODO: docs
    /// not received: dialogs, fullscreen and maximized clients
    fn resize_clients(&self, clients: &mut Vec<&mut Client>);
}
