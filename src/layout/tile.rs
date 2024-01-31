use crate::{
    layout::Layout,
    client::Client,
};

pub struct LayoutTile {}

impl LayoutTile {
    pub fn new() -> Self {
        Self {}
    }
}

impl Layout for LayoutTile {
    fn resize_clients(&self, clients: &mut Vec<&mut Client>) {
        // let border_size = config.border.size;
        //
        // // ....
        // let screen = util::get_screen(conn);
        // let screen_w = screen.width_in_pixels() as u32;
        // let screen_h = screen.height_in_pixels() as u32;
        //
        // let padding_top = clients.iter().map(|c| c.padding.top).max().unwrap_or(0);
        // let padding_bottom = clients.iter().map(|c| c.padding.bottom).max().unwrap_or(0);
        // let padding_left = clients.iter().map(|c| c.padding.left).max().unwrap_or(0);
        // let padding_right = clients.iter().map(|c| c.padding.right).max().unwrap_or(0);
        //
        // // The available wimut dth and height represent the pixels available for drawing windows.
        // // They are the total screen dimensions minus the specified paddings.
        // let available_w = screen_w - padding_left - padding_right;
        // let available_h = screen_h - padding_top - padding_bottom;
        //
        // // Starting tilling at top-right
        // let mut window_x: u32 = config.gap_size;
        // let mut window_y: u32 = config.gap_size + padding_top;
        //
        // // ...
        // let mut window_h: u32 = available_h - (border_size * 2) - (config.gap_size * 2);
        // let mut window_w: u32 = if normal_clients.len() == 1 { 
        //     available_w - (border_size * 2) - (config.gap_size * 2)
        // } else { 
        //     available_w / 2 - border_size - config.gap_size
        // };

        clients
            .iter_mut()
            .for_each(|c| {
                // if i > 0 {
                //     c.rect.w = (available_w / 2) - (border_size * 2) - (config.gap_size * 2);
                //     c.rect.x = available_w / 2 + config.gap_size;
                //
                //     let height_per_window = available_h / (normal_clients.len() - 1) as u32;
                //
                //     c.rect.y = (height_per_window * (i - 1) as u32) + padding_top + config.gap_size;
                //     c.rect.h = if client.id == normal_clients.last().unwrap().id {
                //         height_per_window - (border_size * 2) - (config.gap_size * 2)
                //     } else {
                //         height_per_window - border_size - config.gap_size
                //     };
                // }
            });
    }
}
