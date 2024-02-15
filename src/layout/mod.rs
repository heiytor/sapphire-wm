mod tile;

use crate::{client::Client, tag::TagGeometry};

pub use crate::layout::tile::LayoutTile;

pub trait Layout {
    /// TODO: docs
    /// not received: dialogs, fullscreen and maximized clients
    fn arrange(&self, geometry: TagGeometry, useless_gap: u32, clients: &mut Vec<&mut Client>);
}
