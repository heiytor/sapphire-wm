use crate::{
    client::Client,
    layout::Layout,
    tag::TagGeometry,
};

///  __________   __________
/// |  Master  | |  Window  |
/// |  window  | |          |
/// |          | |          |
/// |          | |__________|
/// |          |  __________
/// |          | |  Window  |
/// |          | |          |
/// |          | |          |
/// |__________| |__________|
pub struct LayoutTile {}

impl LayoutTile {
    pub fn new() -> Self {
        Self {}
    }
}

impl Layout for LayoutTile {
    fn arrange(&self, geometry: TagGeometry, useless_gap: u32, clients: &mut Vec<&mut Client>) {
        let size = clients.len() as u32;

        // gap 6 border 2

        for (i, c) in clients.iter_mut().enumerate() {
            // TODO: padding_left
            c.geo.x = if i == 0 { useless_gap } else { (geometry.avail_w / 2) + useless_gap };
            c.geo.w = (geometry.avail_w / 2) - (useless_gap * 2) - (c.geo.border * 2);

            let mut height_per_window = geometry.avail_h;
            if i != 0 {
                height_per_window /= size - 1
            };

            c.geo.y = (height_per_window * i.checked_sub(1).unwrap_or(0) as u32) + geometry.padding_top() + useless_gap;
            c.geo.h = height_per_window - (c.geo.border * 2) - (useless_gap * 2);
            
            c.geo.x = c.geo.x.max(1);
            c.geo.y = c.geo.y.max(1);
        }
    }
}
